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
    ticket_ids: Vec<u64>,
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

// Implement the 'BoundedStorable' trait for 'Event', 'User', and 'Ticket'
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
    // Retrieve all events from the storage and return them as a Vec
    let events_map: Vec<(u64, Event)> =
        EVENT_STORAGE.with(|events| events.borrow().iter().collect());
    events_map.into_iter().map(|(_, event)| event).collect()
}

#[ic_cdk::query]
fn get_event(id: u64) -> Result<Event, Error> {
    // Retrieve a specific event by ID and return it, or return a NotFound error if not found
    match _get_event(&id) {
        Some(event) => Ok(event),
        None => Err(Error::NotFound {
            msg: format!("event id:{} does not exist", id),
        }),
    }
}

fn _get_event(id: &u64) -> Option<Event> {
    // Helper function to get an event from the storage based on the provided ID
    EVENT_STORAGE.with(|events| events.borrow().get(id))
}

#[ic_cdk::update]
fn create_event(payload: EventPayload) -> Result<Event, Error> {
    // Increment the global ID counter to get a new ID for the event
    let id = ID_COUNTER
        .with(|counter| {
            let current_id = *counter.borrow().get();
            counter.borrow_mut().set(current_id + 1)
        })
        .expect("Cannot increment Ids");

    // Create a new Event with the provided payload and the generated ID
    let event = Event {
        id,
        name: payload.name.clone(),
        description: payload.description,
        date: payload.date,
        start_time: payload.start_time,
        location: payload.location,
        attendee_ids: vec![],
        ticket_ids: vec![],
        created_at: time(),
        updated_at: None,
    };

    // Insert the new event into the storage
    match EVENT_STORAGE.with(|events| events.borrow_mut().insert(id, event.clone())) {
        None => Ok(event),
        Some(_) => Err(Error::NotCreated {
            msg: format!("event {} could not be created", payload.name),
        }),
    }
}

#[ic_cdk::update]
fn update_event(id: u64, payload: EventPayload) -> Result<Event, Error> {
    // Retrieve the existing event with the given ID, or return a NotFound error if not found
    let event = _get_event(&id).ok_or_else(|| Error::NotFound {
        msg: format!("event id:{} does not exist", id),
    })?;

    // Create an updated event based on the provided payload
    let updated_event = Event {
        id,
        name: payload.name,
        description: payload.description,
        date: payload.date,
        start_time: payload.start_time,
        location: payload.location,
        attendee_ids: event.attendee_ids,
        ticket_ids: event.ticket_ids,
        created_at: event.created_at,
        updated_at: Some(time()),
    };

    // Insert the updated event into the storage
    match EVENT_STORAGE.with(|events| events.borrow_mut().insert(id, updated_event.clone())) {
        Some(_) => Ok(updated_event),
        None => Err(Error::NotCreated {
            msg: format!("event id:{} could not be updated", id),
        }),
    }
}


#[ic_cdk::query]
fn get_user(id: u64) -> Result<User, Error> {
    // Retrieve a specific user by ID and return it, or return a NotFound error if not found
    match _get_user(&id) {
        Some(user) => Ok(user),
        None => Err(Error::NotFound {
            msg: format!("user id:{} does not exist", id),
        }),
    }
}

fn _get_user(id: &u64) -> Option<User> {
    // Helper function to get a user from the storage based on the provided ID
    USER_STORAGE.with(|users| users.borrow().get(id))
}

#[ic_cdk::update]
fn create_user(payload: UserPayload) -> Result<User, Error> {
    // Increment the global ID counter to get a new ID for the user
    let id = ID_COUNTER
        .with(|counter| {
            let current_id = *counter.borrow().get();
            counter.borrow_mut().set(current_id + 1)
        })
        .expect("Cannot increment Ids");

    // Create a new User with the provided payload and the generated ID
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

    // Insert the new user into the storage
    match USER_STORAGE.with(|users| users.borrow_mut().insert(id, user.clone())) {
        None => Ok(user),
        Some(_) => Err(Error::NotCreated {
            msg: format!("user id:{} could not be created", id),
        }),
    }
}

#[ic_cdk::update]
fn update_user(id: u64, payload: UserPayload) -> Result<User, Error> {
    // Retrieve the existing user with the given ID, or return a NotFound error if not found
    let user = _get_user(&id).ok_or_else(|| Error::NotFound {
        msg: format!("user id:{} does not exist", id),
    })?;

    // Create an updated user based on the provided payload
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

    // Insert the updated user into the storage
    match USER_STORAGE.with(|users| users.borrow_mut().insert(id, updated_user.clone())) {
        None => Ok(updated_user),
        Some(_) => Err(Error::NotCreated {
            msg: format!("user id:{} could not be updated", id),
        }),
    }
}


#[ic_cdk::query]
fn get_ticket(id: u64) -> Result<Ticket, Error> {
    // Retrieve a specific ticket by ID and return it, or return a NotFound error if not found
    match _get_ticket(&id) {
        Some(ticket) => Ok(ticket),
        None => Err(Error::NotFound {
            msg: format!("ticket id:{} does not exist", id),
        }),
    }
}

fn _get_ticket(id: &u64) -> Option<Ticket> {
    // Helper function to get a ticket from the storage based on the provided ID
    TICKET_STORAGE.with(|tickets| tickets.borrow().get(id))
}

#[ic_cdk::update]
fn create_ticket(payload: TicketPayload) -> Result<Ticket, AssociationError> {
    // Increment the global ID counter to get a new ID for the ticket
    let id = ID_COUNTER
        .with(|counter| {
            let current_id = *counter.borrow().get();
            counter.borrow_mut().set(current_id + 1)
        })
        .expect("Cannot increment Ids");

    // Create a new Ticket with the provided payload and the generated ID
    let ticket = Ticket {
        id,
        event_id: payload.event_id,
        user_id: payload.user_id,
        created_at: time(),
        updated_at: None,
    };

    // Insert the new ticket into the storage
    TICKET_STORAGE.with(|tickets| tickets.borrow_mut().insert(id, ticket.clone()));

    // Call helper functions to associate the ticket with the event and user
    match add_event_attendee(payload.event_id, payload.user_id) {
        Ok(_) => (),
        Err(_) => {
            TICKET_STORAGE.with(|tickets| tickets.borrow_mut().remove(&id));
            return Err(AssociationError::Err {
                msg: format!("Could not add attendee to event id:{} ", payload.event_id),
                ticket: ticket.clone(),
            })
        }
    }

    match add_user_ticket(payload.user_id, id) {
        Ok(_) => (),
        Err(_) => {
            TICKET_STORAGE.with(|tickets| tickets.borrow_mut().remove(&id));
            return Err(AssociationError::Err {
                msg: format!(
                    "Could not add ticket id:{} to user id:{} ",
                    id, payload.user_id
                ),
                ticket: ticket.clone(),
            })
        }
    }

    match add_event_ticket(payload.event_id, id) {
        Ok(_) => (),
        Err(_) => {
            return Err(AssociationError::Err {
                msg: format!(
                    "Could not add ticket id:{} to event id:{} ",
                    id, payload.event_id
                ),
                ticket: ticket.clone(),
            })
        }
    }

    // Return the ID of the newly created ticket
    Ok(ticket)
}

#[ic_cdk::update]
fn delete_ticket(id: u64) -> Result<String, Error> {
    // Retrieve the ticket ID from the payload
    let ticket_id = id;

    // Retrieve the ticket with the given ID, or return a NotFound error if not found
    let ticket = _get_ticket(&ticket_id).ok_or_else(|| Error::NotFound {
        msg: format!("ticket id:{} does not exist", ticket_id),
    })?;

    // Retrieve the user with the given ID, or return a NotFound error if not found
    let user_id = ticket.user_id;
    let mut user = _get_user(&user_id).ok_or_else(|| Error::NotFound {
        msg: format!("user id:{} does not exist", user_id),
    })?;

    // Retrieve the event with the given ID, or return a NotFound error if not found
    let event_id = ticket.event_id;
    let mut event = _get_event(&event_id).ok_or_else(|| Error::NotFound {
        msg: format!("event id:{} does not exist", event_id),
    })?;


    // Remove the ticket ID from the user's ticket IDs
    user.ticket_ids.retain(|&id| id != ticket_id);

    // Remove the ticket ID from the event's ticket IDs
    event.ticket_ids.retain(|&id| id != ticket_id);

    // Update the user in the storage
    match USER_STORAGE.with(|users| users.borrow_mut().insert(user_id, user)) {
        Some(_) => (),
        None => {
            return Err(Error::NotFound {
                msg: format!("user id:{} could not be updated", user_id),
            })
        }
    }

    // Update the event in the storage
    match EVENT_STORAGE.with(|events| events.borrow_mut().insert(event_id, event)) {
        Some(_) => (),
        None => {
            return Err(Error::NotFound {
                msg: format!("event id:{} could not be updated", event_id),
            })
        }
    }
    // Delete the ticket from the storage
    match TICKET_STORAGE.with(|tickets| tickets.borrow_mut().remove(&ticket_id)) {
        Some(_) => (),
        None => {
            return Err(Error::NotFound {
                msg: format!("ticket id:{} could not be deleted from event", ticket_id),
            })
        }
    }
    // Return Ok indicating a successful deletion
    Ok(format!("ticket id: {} deleted", ticket_id))
}

#[ic_cdk::query]
fn get_event_attendees(id: u64) -> Result<Vec<User>, Error> {
    // Retrieve the event with the given ID, or return a NotFound error if not found
    let event = _get_event(&id).ok_or_else(|| Error::NotFound {
        msg: format!("event id:{} does not exist", id),
    })?;

    // Initialize a vector to store the attendees
    let mut attendees = vec![];

    // Iterate over the attendee IDs of the event and retrieve the corresponding users
    for attendee_id in event.attendee_ids {
        let attendee = _get_user(&attendee_id).ok_or_else(|| Error::NotFound {
            msg: format!("user id:{} does not exist", attendee_id),
        })?;

        // Add the attendee to the vector
        attendees.push(attendee);
    }

    // Return the vector of attendees
    Ok(attendees)
}

// Function to add an attendee to an event
fn add_event_attendee(event_id: u64, user_id: u64) -> Result<(), Error> {
    // Retrieve the event with the given ID, or return a NotFound error if not found
    let event = _get_event(&event_id).ok_or_else(|| Error::NotFound {
        msg: format!("event id:{} does not exist", event_id),
    })?;

    // Retrieve the user with the given ID, or return a NotFound error if not found
    let user = _get_user(&user_id).ok_or_else(|| Error::NotFound {
        msg: format!("user id:{} does not exist", user_id),
    })?;

    // Clone the current attendee IDs and add the new user ID
    let mut attendees = event.attendee_ids.clone();
    attendees.push(user.id);

    // Create an updated event with the new attendee IDs
    let updated_event = Event {
        id: event.id,
        name: event.name,
        description: event.description,
        date: event.date,
        start_time: event.start_time,
        location: event.location,
        attendee_ids: attendees,
        ticket_ids: event.ticket_ids,
        created_at: event.created_at,
        updated_at: Some(time()),
    };

    // Update the event in the storage
    EVENT_STORAGE.with(|events| events.borrow_mut().insert(event.id, updated_event));

    // Return Ok indicating a successful update
    Ok(())
}

// Function to add a ticket to an event
fn add_event_ticket(event_id: u64, ticket_id: u64) -> Result<(), Error> {
    // Retrieve the event with the given ID, or return a NotFound error if not found
    let event = _get_event(&event_id).ok_or_else(|| Error::NotFound {
        msg: format!("event id:{} does not exist", event_id),
    })?;

    // Retrieve the ticket with the given ID, or return a NotFound error if not found
    let ticket = _get_ticket(&ticket_id).ok_or_else(|| Error::NotFound {
        msg: format!("ticket id:{} does not exist", ticket_id),
    })?;

    // Clone the current ticket IDs and add the new ticket ID
    let mut tickets = event.ticket_ids.clone();
    tickets.push(ticket.id);

    // Create an updated event with the new ticket IDs
    let updated_event = Event {
        id: event.id,
        name: event.name,
        description: event.description,
        date: event.date,
        start_time: event.start_time,
        location: event.location,
        attendee_ids: event.attendee_ids,
        ticket_ids: tickets,
        created_at: event.created_at,
        updated_at: Some(time()),
    };

    // Update the event in the storage
    EVENT_STORAGE.with(|events| events.borrow_mut().insert(event.id, updated_event));

    // Return Ok indicating a successful update
    Ok(())
}

#[ic_cdk::query]
fn get_user_tickets(id: u64) -> Result<Vec<Ticket>, Error> {
    // Retrieve the user with the given ID, or return a NotFound error if not found
    let user = _get_user(&id).ok_or_else(|| Error::NotFound {
        msg: format!("user id:{} does not exist", id),
    })?;

    // Initialize a vector to store the user's tickets
    let mut tickets = vec![];

    // Iterate over the ticket IDs of the user and retrieve the corresponding tickets
    for ticket_id in user.ticket_ids {
        let ticket = _get_ticket(&ticket_id).ok_or_else(|| Error::NotFound {
            msg: format!("ticket id:{} does not exist", ticket_id),
        })?;

        // Add the ticket to the vector
        tickets.push(ticket);
    }

    // Return the vector of tickets
    match tickets.len() {
        0 => Err(Error::NotFound {
            msg: format!("event id:{} has no tickets", id),
        }),
        _ => Ok(tickets),
    }
}

#[ic_cdk::query]
fn get_event_tickets(id: u64) -> Result<Vec<Ticket>, Error> {
    // Retrieve the event with the given ID, or return a NotFound error if not found
    let event = _get_event(&id).ok_or_else(|| Error::NotFound {
        msg: format!("event id:{} does not exist", id),
    })?;

    // Initialize a vector to store the event's tickets
    let mut tickets = vec![];

    // Iterate over the ticket IDs of the event and retrieve the corresponding tickets
    for ticket_id in event.ticket_ids {
        let ticket = _get_ticket(&ticket_id).ok_or_else(|| Error::NotFound {
            msg: format!("ticket id:{} does not exist", ticket_id),
        })?;

        // Add the ticket to the vector
        tickets.push(ticket);
    }

    // Return the vector of tickets
    Ok(tickets)
}

// Function to add a ticket to a user's tickets
fn add_user_ticket(user_id: u64, ticket_id: u64) -> Result<(), Error> {
    // Retrieve the user with the given ID, or return a NotFound error if not found
    let user = _get_user(&user_id).ok_or_else(|| Error::NotFound {
        msg: format!("user id:{} does not exist", user_id),
    })?;

    // Retrieve the ticket with the given ID, or return a NotFound error if not found
    let ticket = _get_ticket(&ticket_id).ok_or_else(|| Error::NotFound {
        msg: format!("ticket id:{} does not exist", ticket_id),
    })?;

    // Clone the current ticket IDs and add the new ticket ID
    let mut tickets = user.ticket_ids.clone();
    tickets.push(ticket.id);

    // Create an updated user with the new ticket IDs
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

    // Update the user in the storage
    USER_STORAGE.with(|users| users.borrow_mut().insert(user.id, updated_user));

    // Return Ok indicating a successful update
    Ok(())
}


// Define an Error enum for handling errors
#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
    NotCreated { msg: String },
}

// Define an Error enum for handling errors
#[derive(candid::CandidType, Deserialize, Serialize)]
enum AssociationError {
    Err { msg: String, ticket: Ticket },
}

// Candid generator for exporting the Candid interface
ic_cdk::export_candid!();
