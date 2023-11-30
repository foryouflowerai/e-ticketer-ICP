#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};

// Define type aliases for convenience
type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

// Define a struct for the 'Event'
#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Event {
    id: u64,
    name: String,
    description: String,
    date: String,
    start_time: String,
    location: String,
    attendee_ids: Vec<u64>,
    created_at: u64,
    updated_at: Option<u64>,
}

// Define a struct for the 'User'
#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct User {
    id: u64,
    name: String,
    email: String,
    password: String,
    event_ids: Vec<u64>,
    ticket_ids: Vec<u64>,
    created_at: u64,
    updated_at: Option<u64>,
}

// Define a struct for the 'Ticket'
#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Ticket {
    id: u64,
    event_id: u64,
    user_id: u64,
    created_at: u64,
    updated_at: Option<u64>,
}

// Implement the 'Storable' trait for 'Event', 'User', and 'Ticket'
impl Storable for Event {
    // Conversion to bytes
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }
    // Conversion from bytes
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl Storable for User {
    // Conversion to bytes
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }
    // Conversion from bytes
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl Storable for Ticket {
    // Conversion to bytes
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }
    // Conversion from bytes
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

// Implement the 'BoundedStorable' trait for 'Event', 'User', and 'Event'
impl BoundedStorable for Event {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

impl BoundedStorable for User {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

impl BoundedStorable for Ticket {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

// Define thread-local static variables for memory management and storage
thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static EVENT_STORAGE: RefCell<StableBTreeMap<u64, Event, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
    ));

    static USER_STORAGE: RefCell<StableBTreeMap<u64, User, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
    ));

    static TICKET_STORAGE: RefCell<StableBTreeMap<u64, Ticket, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3)))
    ));
}

// Define structs for payload data (used in update calls)
#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct EventPayload {
    name: String,
    description: String,
    date: String,
    start_time: String,
    location: String,
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct UserPayload {
    name: String,
    email: String,
    password: String,
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct TicketPayload {
    event_id: u64,
    user_id: u64,
}

// Define the Candid interface
#[ic_cdk::query]
fn get_all_events() -> Vec<Event> {
    let events_map: Vec<(u64, Event)> =
        EVENT_STORAGE.with(|events| events.borrow().iter().collect());
    events_map.into_iter().map(|(_, event)| event).collect()
}

#[ic_cdk::query]
fn get_event(id: u64) -> Result<Event, Error> {
    match _get_event(&id) {
        Some(event) => Ok(event),
        None => Err(Error::NotFound {
            msg: format!("event id:{} does not exist", id),
        }),
    }
}

fn _get_event(id: &u64) -> Option<Event> {
    EVENT_STORAGE.with(|events| events.borrow().get(id))
}

#[ic_cdk::update]
fn create_event(payload: EventPayload) -> u64 {
    // Increment the global ID counter to get a new ID for the supplier
    let id = ID_COUNTER
        .with(|counter| {
            let current_id = *counter.borrow().get();
            counter.borrow_mut().set(current_id + 1)
        })
        .expect("Cannot increment Ids");

    let event = Event {
        id,
        name: payload.name,
        description: payload.description,
        date: payload.date,
        start_time: payload.start_time,
        location: payload.location,
        attendee_ids: vec![],
        created_at: time(),
        updated_at: None,
    };

    EVENT_STORAGE.with(|events| events.borrow_mut().insert(id, event));

    id
}

#[ic_cdk::update]
fn update_event(id: u64, payload: EventPayload) -> Result<(), Error> {
    let event = _get_event(&id).ok_or(Error::NotFound {
        msg: format!("event id:{} does not exist", id),
    })?;

    let updated_event = Event {
        id,
        name: payload.name,
        description: payload.description,
        date: payload.date,
        start_time: payload.start_time,
        location: payload.location,
        attendee_ids: event.attendee_ids,
        created_at: event.created_at,
        updated_at: Some(time()),
    };

    EVENT_STORAGE.with(|events| events.borrow_mut().insert(id, updated_event));

    Ok(())
}

#[ic_cdk::update]
fn delete_event(id: u64) -> Result<(), Error> {
    _get_event(&id).ok_or(Error::NotFound {
        msg: format!("event id:{} does not exist", id),
    })?;

    EVENT_STORAGE.with(|events| events.borrow_mut().remove(&id));

    Ok(())
}

#[ic_cdk::query]
fn get_user(id: u64) -> Result<User, Error> {
    match _get_user(&id) {
        Some(user) => Ok(user),
        None => Err(Error::NotFound {
            msg: format!("user id:{} does not exist", id),
        }),
    }
}

fn _get_user(id: &u64) -> Option<User> {
    USER_STORAGE.with(|users| users.borrow().get(id))
}

#[ic_cdk::update]
fn create_user(payload: UserPayload) -> u64 {
    // Increment the global ID counter to get a new ID for the supplier
    let id = ID_COUNTER
        .with(|counter| {
            let current_id = *counter.borrow().get();
            counter.borrow_mut().set(current_id + 1)
        })
        .expect("Cannot increment Ids");

    let user = User {
        id,
        name: payload.name,
        email: payload.email,
        password: payload.password,
        event_ids: vec![],
        ticket_ids: vec![],
        created_at: time(),
        updated_at: None,
    };

    USER_STORAGE.with(|users| users.borrow_mut().insert(id, user));

    id
}

#[ic_cdk::update]
fn update_user(id: u64, payload: UserPayload) -> Result<(), Error> {
    let user = _get_user(&id).ok_or(Error::NotFound {
        msg: format!("user id:{} does not exist", id),
    })?;

    let updated_user = User {
        id,
        name: payload.name,
        email: payload.email,
        password: payload.password,
        event_ids: user.event_ids,
        ticket_ids: user.ticket_ids,
        created_at: user.created_at,
        updated_at: Some(time()),
    };

    USER_STORAGE.with(|users| users.borrow_mut().insert(id, updated_user));

    Ok(())
}

#[ic_cdk::update]
fn delete_user(id: u64) -> Result<(), Error> {
    _get_user(&id).ok_or(Error::NotFound {
        msg: format!("user id:{} does not exist", id),
    })?;

    USER_STORAGE.with(|users| users.borrow_mut().remove(&id));

    Ok(())
}

#[ic_cdk::query]
fn get_ticket(id: u64) -> Result<Ticket, Error> {
    match _get_ticket(&id) {
        Some(ticket) => Ok(ticket),
        None => Err(Error::NotFound {
            msg: format!("ticket id:{} does not exist", id),
        }),
    }
}

fn _get_ticket(id: &u64) -> Option<Ticket> {
    TICKET_STORAGE.with(|tickets| tickets.borrow().get(id))
}

#[ic_cdk::update]
fn create_ticket(payload: TicketPayload) -> u64 {
    // Increment the global ID counter to get a new ID for the supplier
    let id = ID_COUNTER
        .with(|counter| {
            let current_id = *counter.borrow().get();
            counter.borrow_mut().set(current_id + 1)
        })
        .expect("Cannot increment Ids");

    let ticket = Ticket {
        id,
        event_id: payload.event_id,
        user_id: payload.user_id,
        created_at: time(),
        updated_at: None,
    };

    TICKET_STORAGE.with(|tickets| tickets.borrow_mut().insert(id, ticket));
    let _ = add_event_attendee(payload.event_id, payload.user_id);
    let _ = add_user_ticket(payload.user_id, id);

    id
}

#[ic_cdk::update]
fn update_ticket(id: u64, payload: TicketPayload) -> Result<(), Error> {
    let ticket = _get_ticket(&id).ok_or(Error::NotFound {
        msg: format!("ticket id:{} does not exist", id),
    })?;

    let updated_ticket = Ticket {
        id,
        event_id: payload.event_id,
        user_id: payload.user_id,
        created_at: ticket.created_at,
        updated_at: Some(time()),
    };

    TICKET_STORAGE.with(|tickets| tickets.borrow_mut().insert(id, updated_ticket));

    Ok(())
}

#[ic_cdk::update]
fn delete_ticket(id: u64) -> Result<(), Error> {
    _get_ticket(&id).ok_or(Error::NotFound {
        msg: format!("ticket id:{} does not exist", id),
    })?;

    TICKET_STORAGE.with(|tickets| tickets.borrow_mut().remove(&id));

    Ok(())
}

#[ic_cdk::query]
fn get_event_attendees(id: u64) -> Result<Vec<User>, Error> {
    let event = _get_event(&id).ok_or(Error::NotFound {
        msg: format!("event id:{} does not exist", id),
    })?;

    let mut attendees = vec![];

    for attendee_id in event.attendee_ids {
        let attendee = _get_user(&attendee_id).ok_or(Error::NotFound {
            msg: format!("user id:{} does not exist", attendee_id),
        })?;

        attendees.push(attendee);
    }

    Ok(attendees)
}

fn add_event_attendee(event_id: u64, user_id: u64) -> Result<(), Error> {
    let event = _get_event(&event_id).ok_or(Error::NotFound {
        msg: format!("event id:{} does not exist", event_id),
    })?;

    let user = _get_user(&user_id).ok_or(Error::NotFound {
        msg: format!("user id:{} does not exist", user_id),
    })?;

    let mut attendees = event.attendee_ids.clone();
    attendees.push(user.id);

    let updated_event = Event {
        id: event.id,
        name: event.name,
        description: event.description,
        date: event.date,
        start_time: event.start_time,
        location: event.location,
        attendee_ids: attendees,
        created_at: event.created_at,
        updated_at: Some(time()),
    };

    EVENT_STORAGE.with(|events| events.borrow_mut().insert(event.id, updated_event));

    Ok(())
}

#[ic_cdk::query]
fn get_user_tickets(id: u64) -> Result<Vec<Ticket>, Error> {
    let user = _get_user(&id).ok_or(Error::NotFound {
        msg: format!("user id:{} does not exist", id),
    })?;

    let mut tickets = vec![];

    for ticket_id in user.ticket_ids {
        let ticket = _get_ticket(&ticket_id).ok_or(Error::NotFound {
            msg: format!("ticket id:{} does not exist", ticket_id),
        })?;

        tickets.push(ticket);
    }

    Ok(tickets)
}

fn add_user_ticket(user_id: u64, ticket_id: u64) -> Result<(), Error> {
    let user = _get_user(&user_id).ok_or(Error::NotFound {
        msg: format!("user id:{} does not exist", user_id),
    })?;

    let ticket = _get_ticket(&ticket_id).ok_or(Error::NotFound {
        msg: format!("ticket id:{} does not exist", ticket_id),
    })?;

    let mut tickets = user.ticket_ids.clone();
    tickets.push(ticket.id);

    let updated_user = User {
        id: user.id,
        name: user.name,
        email: user.email,
        password: user.password,
        event_ids: user.event_ids,
        ticket_ids: tickets,
        created_at: user.created_at,
        updated_at: Some(time()),
    };

    USER_STORAGE.with(|users| users.borrow_mut().insert(user.id, updated_user));

    Ok(())
}

#[ic_cdk::update]
fn remove_user_ticket(payload: TicketPayload) -> Result<(), Error> {
    let ticket_id = payload.event_id;
    let user_id = payload.user_id;
    let user = _get_user(&user_id).ok_or(Error::NotFound {
        msg: format!("user id:{} does not exist", user_id),
    })?;

    let ticket = _get_ticket(&ticket_id).ok_or(Error::NotFound {
        msg: format!("ticket id:{} does not exist", ticket_id),
    })?;

    let mut tickets = user.ticket_ids.clone();
    tickets.retain(|&ticket_id| ticket_id != ticket.id);

    let updated_user = User {
        id: user.id,
        name: user.name,
        email: user.email,
        password: user.password,
        event_ids: user.event_ids,
        ticket_ids: tickets,
        created_at: user.created_at,
        updated_at: Some(time()),
    };

    USER_STORAGE.with(|users| users.borrow_mut().insert(user.id, updated_user));

    Ok(())
}

// Define an Error enum for handling errors
#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
}

// Candid generator for exporting the Candid interface
ic_cdk::export_candid!();
