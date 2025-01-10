#[macro_use]
extern crate serde;
use candid::{CandidType, Decode, Encode, Principal};
use ic_cdk::api::caller;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

/**
 * Enumerations
 */

// Servive Categories
#[derive(CandidType, Clone, Serialize, Deserialize, PartialEq, Default, Debug)]
enum Category {
    #[default]
    Venue,
    Catering,
    Photography,
    Music,
    Decor,
    Planning,
    Attire,
    Beauty,
    Transport,
    Stationery,
    Cake,
    Favors,
    Other,
}

// Table Assignment Enum
#[derive(CandidType, Clone, Serialize, Deserialize, Default, Debug)]
enum TableAssignment {
    #[default]
    VIPTable,
    FamilyTable,
    Table(u8), // e.g., Table(1), Table(2), Table(3), etc.
    Unassigned,
}

/**
 * Core Types
 */

// Review Record
#[derive(CandidType, Clone, Serialize, Deserialize, Debug)]
struct Review {
    author: Principal,
    rating: u64, // 1-10
    comment: String,
    date: String,
}

// Vendor Details Record
#[derive(CandidType, Clone, Serialize, Deserialize, Debug)]
struct Vendor {
    id: u64,
    owner: Principal, // Principal ID as string
    name: String,
    category: Category,
    description: String,
    service_cost: u64,
    availability: Vec<String>,
    rating: u64,
    reviews: Vec<Review>,
    bookings: Vec<String>, // Wedding IDs
    verified: bool,
    portfolio: Vec<String>,
}

// Vendor Booking Record
#[derive(CandidType, Clone, Serialize, Deserialize, Default)]
struct VendorBooking {
    vendor_id: u64,
    wedding_id: u64,
    wedding_offer: u64,
    additional_details: Option<String>,
    status: String, // e.g., "pending", "confirmed", "rejected", "paid"
    date: String,
}

// Timeline Item Record
#[derive(CandidType, Clone, Serialize, Deserialize, Default)]
struct TimelineItem {
    wedding_id: u64,
    time: String,
    description: String,
    responsible: String,
    status: String, // e.g., "pending", "completed", "overdue"
}

// Task Record
#[derive(CandidType, Clone, Serialize, Deserialize, Default)]
struct Task {
    id: u64,
    title: String,
    description: String,
    deadline: String,
    assigned_to: String,
    status: String, // e.g "pending", "in-progress", "completed"
    budget: u64,
}

// Guest Details Record
#[derive(CandidType, Clone, Serialize, Deserialize, Default)]
struct Guest {
    name: String,
    guest_email: String,
    rsvp_status: String, // e.g "pending", "confirmed", "declined"
    dietary_restrictions: String,
    plus_one: bool,
    table_assignment: TableAssignment,
}

// Registry Item Record
#[derive(CandidType, Clone, Serialize, Deserialize, Default)]
struct RegistryItem {
    name: String,
    description: String,
    price: u64,
    status: String, // "available", "purchased"
    purchased_by: String,
}

// Wedding Record with all details
#[derive(CandidType, Clone, Serialize, Deserialize, Default)]
struct Wedding {
    id: u64,
    couple_names: Vec<String>,
    date: String,
    budget: u64,
    location: String,
    guest_count: u64,
    vendors: Vec<VendorBooking>,
    timeline: Vec<TimelineItem>,
    tasks: Vec<Task>,
    guest_list: Vec<Guest>,
    registry: Vec<RegistryItem>,
    status: String, // "planning", "upcoming", "completed"
}

// Implement Storable and BoundedStorable for all types
impl Storable for Wedding {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Wedding {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

impl Storable for Vendor {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Vendor {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

// Thread local storage
thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static VENDOR_STORAGE: RefCell<StableBTreeMap<u64, Vendor, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))))
    );


    static WEDDING_STORAGE: RefCell<StableBTreeMap<u64, Wedding, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2))))
    );
}

/**
 * Payload Definitions
 */

// Vendor Management Payload
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
struct RegisterVendorPayload {
    name: String,
    category: Category,
    description: String,
    service_cost: u64,
    availability: Vec<String>,
    portfolio: Vec<String>,
}

// Wedding Planning Payload
#[derive(CandidType, Serialize, Deserialize)]
struct CreateWeddingPayload {
    couple_names: Vec<String>,
    date: String,
    budget: u64,
    location: String,
    guest_count: u64,
}

// GuestRSVP Payload
#[derive(CandidType, Serialize, Deserialize)]
struct GuestRsvpPayload {
    wedding_id: u64,
    name: String,
    guest_email: String,
    dietary_restrictions: String,
    plus_one: bool,
}

// Approve RSVP Payload
#[derive(CandidType, Serialize, Deserialize)]
struct ApproveRsvpPayload {
    wedding_id: u64,
    guest_email: String,
    table_assignment: TableAssignment,
}

// Vendor Booking
#[derive(CandidType, Serialize, Deserialize)]
struct VendorBookingPayload {
    vendor_id: u64,
    wedding_id: u64,
    wedding_offer: u64,
    additional_details: Option<String>,
}

// selectVendorForService Payload
#[derive(CandidType, Serialize, Deserialize)]
struct SelectVendorForServicePayload {
    wedding_id: u64,
    vendor_id: u64,
    category: Category,
}

// Timeline Item Payload
#[derive(CandidType, Serialize, Deserialize)]
struct TimelineItemPayload {
    wedding_id: u64,
    time: String,
    description: String,
    responsible: String,
    status: String,
}

// Task Payload
#[derive(CandidType, Serialize, Deserialize)]
struct TaskPayload {
    wedding_id: u64,
    title: String,
    description: String,
    deadline: String,
    assigned_to: String,
    budget: u64,
}

// Update Task Status Payload
#[derive(CandidType, Serialize, Deserialize)]
struct UpdateTaskStatusPayload {
    wedding_id: u64,
    task_id: u64,
    status: String,
}

// Delete Task Payload
#[derive(CandidType, Serialize, Deserialize)]
struct DeleteTaskPayload {
    wedding_id: u64,
    task_id: u64,
}

// Add Registry Item Payload
#[derive(CandidType, Serialize, Deserialize)]
struct AddRegistryItemPayload {
    wedding_id: u64,
    name: String,
    description: String,
    price: u64,
}

// Update Registry Item Status Payload
#[derive(CandidType, Serialize, Deserialize)]
struct UpdateRegistryItemStatusPayload {
    wedding_id: u64,
    item_name: String,
    status: String,
    purchased_by: String,
}

// Delete Registry Item Payload
#[derive(CandidType, Serialize, Deserialize)]
struct DeleteRegistryItemPayload {
    wedding_id: u64,
    item_name: String,
}

// Result type for error handling
#[derive(CandidType, Serialize, Deserialize)]
enum Message {
    Success(String),
    Error(String),
    InvalidInput(String),
    VendorNotFound(String),
    WeddingNotFound(String),
    NoTimeLineItemsFound(String),
    DateUnavailable(String),
    UnauthorizedAction(String),
    BudgetExceeded(String),
    InvalidDate(String),
}

/**
 * Helper Functions
 */

// Generate UUID
fn generate_uuid() -> u64 {
    ID_COUNTER.with(|counter| {
        let mut counter = counter.borrow_mut();
        let current_id = *counter.get(); // Dereference the reference to get the value
        counter.set(current_id + 1).expect("Failed to update ID counter");
        current_id
    })
}

/**
 * Canister Definition - Implementation of core functions
 */

/**
 * Vendor Management
 */

#[ic_cdk::update]
fn register_vendor(payload: RegisterVendorPayload) -> Result<(Vendor, Message), Message> {
    if payload.name.is_empty() || payload.description.is_empty() || payload.service_cost == 0 {
        return Err(Message::InvalidInput(
            "Name, description, and service cost are required.".to_string(),
        ));
    }

    let vendor_id = generate_uuid();

    let vendor = Vendor {
        id: vendor_id,
        owner: caller(),
        name: payload.name,
        category: payload.category,
        description: payload.description,
        service_cost: payload.service_cost,
        availability: payload.availability,
        rating: 0,
        reviews: Vec::new(),
        bookings: Vec::new(),
        verified: false,
        portfolio: payload.portfolio,
    };

    VENDOR_STORAGE.with(|vendors| {
        vendors.borrow_mut().insert(vendor_id, vendor.clone());
    });

    Ok((
        vendor,
        Message::Success("Vendor registered successfully".to_string()),
    ))
}


// Book Vendor for Wedding
#[ic_cdk::update]
fn book_vendor(
    payload: VendorBookingPayload,
) -> Result<(String, Wedding, Vendor, VendorBooking), Message> {
    // Fetch the wedding and vendor from storage
    let wedding = WEDDING_STORAGE.with(|storage| storage.borrow().get(&payload.wedding_id));
    let vendor = VENDOR_STORAGE.with(|storage| storage.borrow().get(&payload.vendor_id));

    // Validate wedding existence
    let wedding = match wedding {
        Some(wedding) => wedding.clone(),
        None => {
            return Err(Message::WeddingNotFound(format!(
                "Wedding with ID {} not found",
                payload.wedding_id
            )))
        }
    };

    // Validate vendor existence
    let vendor = match vendor {
        Some(vendor) => vendor.clone(),
        None => {
            return Err(Message::VendorNotFound(format!(
                "Vendor with ID {} not found",
                payload.vendor_id
            )))
        }
    };

    // Check vendor availability
    if !vendor.availability.iter().any(|date| date == &wedding.date) {
        return Err(Message::DateUnavailable(format!(
            "Vendor not available on wedding date {}",
            wedding.date
        )));
    }

    // Create the vendor booking
    let vendor_booking = VendorBooking {
        vendor_id: payload.vendor_id,
        wedding_id: payload.wedding_id,
        wedding_offer: payload.wedding_offer,
        additional_details: payload.additional_details.clone(),
        status: "pending".to_string(),
        date: wedding.date.clone(),
    };

    // Update the wedding with the new vendor booking
    let mut updated_wedding = wedding.clone();
    updated_wedding.vendors.push(vendor_booking.clone());

    WEDDING_STORAGE.with(|storage| {
        storage
            .borrow_mut()
            .insert(payload.wedding_id, updated_wedding.clone());
    });

    // Update the vendor with the new booking
    let mut updated_vendor = vendor.clone();
    updated_vendor.bookings.push(wedding.id.to_string());

    VENDOR_STORAGE.with(|storage| {
        storage
            .borrow_mut()
            .insert(payload.vendor_id, updated_vendor.clone());
    });

    // Return success
    Ok((
        "Vendor booked successfully".to_string(),
        updated_wedding,
        vendor,
        vendor_booking,
    ))
}

// Verify Vendor Booking done by the Vendor
#[ic_cdk::update]
fn verify_vendor_booking(
    vendor_id: u64,
    wedding_id: u64,
) -> Result<(VendorBooking, Message), Message> {
    // Fetch the vendor from storage
    let vendor = VENDOR_STORAGE.with(|storage| storage.borrow().get(&vendor_id));

    // Validate vendor existence
    let vendor = match vendor {
        Some(vendor) => vendor.clone(),
        None => {
            return Err(Message::VendorNotFound(format!(
                "Vendor with ID {} not found",
                vendor_id
            )))
        }
    };

    if vendor.owner != caller() {
        return Err(Message::UnauthorizedAction(
            "You are not authorized to perform this action".to_string(),
        ));
    }

    // Fetch the wedding from storage
    let wedding = WEDDING_STORAGE.with(|storage| storage.borrow().get(&wedding_id));

    // Validate wedding existence
    let wedding = match wedding {
        Some(wedding) => wedding.clone(),
        None => {
            return Err(Message::WeddingNotFound(format!(
                "Wedding with ID {} not found",
                wedding_id
            )))
        }
    };

    // Fetch the vendor booking from the wedding
    let booking = wedding
        .vendors
        .iter()
        .find(|booking| booking.vendor_id == vendor_id);

    let booking = match booking {
        Some(booking) => booking.clone(),
        None => {
            return Err(Message::Error("Vendor booking not found".to_string()));
        }
    };

    // Update the vendor booking status
    let mut updated_booking = booking.clone();
    updated_booking.status = "confirmed".to_string();

    // Save the updated vendor booking details
    WEDDING_STORAGE.with(|storage| {
        let mut updated_wedding = wedding.clone();
        updated_wedding.vendors = updated_wedding
            .vendors
            .iter()
            .map(|booking| {
                if booking.vendor_id == vendor_id {
                    updated_booking.clone()
                } else {
                    booking.clone()
                }
            })
            .collect();

        storage
            .borrow_mut()
            .insert(wedding_id, updated_wedding.clone());
    });

    // Return success
    Ok((
        updated_booking,
        Message::Success("Vendor booking verified successfully".to_string()),
    ))
}

// Verify Vendor
#[ic_cdk::update]
fn verify_vendor(vendor_id: u64) -> Result<(Vendor, Message), Message> {
    // Fetch the vendor from storage
    let vendor = VENDOR_STORAGE.with(|storage| storage.borrow().get(&vendor_id));

    // Validate vendor existence
    let vendor = match vendor {
        Some(vendor) => vendor.clone(),
        None => {
            return Err(Message::VendorNotFound(format!(
                "Vendor with ID {} not found",
                vendor_id
            )))
        }
    };

    // Update the vendor's verified status
    let mut updated_vendor = vendor.clone();
    updated_vendor.verified = true;

    // Save the updated vendor details
    VENDOR_STORAGE.with(|storage| {
        storage
            .borrow_mut()
            .insert(vendor_id, updated_vendor.clone());
    });

    // Return success
    Ok((
        updated_vendor,
        Message::Success("Vendor verified successfully".to_string()),
    ))
}

/**
 * Vendor Queries
 */

// Get Vendor Details by ID
#[ic_cdk::query]
fn get_vendor_details(vendor_id: u64) -> Result<Vendor, Message> {
    VENDOR_STORAGE.with(|vendors| match vendors.borrow().get(&vendor_id) {
        Some(vendor) => Ok(vendor.clone()),
        None => Err(Message::VendorNotFound(format!(
            "Vendor with id={} not found",
            vendor_id
        ))),
    })
}

// Get Vendors by Category
#[ic_cdk::query]
fn search_vendors_by_category(category: Category) -> Result<Vec<Vendor>, Message> {
    let matching_vendors: Vec<Vendor> = VENDOR_STORAGE.with(|vendors| {
        vendors
            .borrow()
            .iter()
            .filter_map(|(_, vendor)| {
                if vendor.category == category {
                    Some(vendor.clone()) // Include matching vendor
                } else {
                    None // Skip non-matching vendor
                }
            })
            .collect() // Collect all matching vendors into a Vec
    });

    if matching_vendors.is_empty() {
        Err(Message::VendorNotFound(format!(
            "No vendors found in the '{}' category",
            format!("{:?}", category) // Convert the enum to a readable string
        )))
    } else {
        Ok(matching_vendors)
    }
}

// Get All Vendors
#[ic_cdk::query]
fn get_all_vendors() -> Result<Vec<Vendor>, Message> {
    VENDOR_STORAGE.with(|vendors| {
        let all_vendors: Vec<Vendor> = vendors
            .borrow()
            .iter()
            .map(|(_, vendor)| vendor.clone()) // Clone each vendor to return owned data
            .collect();

        if all_vendors.is_empty() {
            Err(Message::VendorNotFound("No vendors found.".to_string()))
        } else {
            Ok(all_vendors)
        }
    })
}

/**
 * Wedding Management
 */
#[ic_cdk::update]
fn create_wedding(payload: CreateWeddingPayload) -> Result<(Wedding, Message), Message> {
    // Validate the user input to ensure all required fields are provided
    if payload.couple_names.is_empty()
        && payload.date.is_empty()
        && payload.budget == 0
        && payload.location.is_empty()
        && payload.guest_count == 0
    {
        return Err(Message::InvalidInput(
            "Ensure all required fileds are provided.".to_string(),
        ));
    }

    // Generate a unique ID for the wedding
    let wedding_id = generate_uuid();

    let wedding = Wedding {
        id: wedding_id,
        couple_names: payload.couple_names,
        date: payload.date,
        budget: payload.budget,
        location: payload.location,
        guest_count: payload.guest_count,
        vendors: Vec::new(),
        timeline: Vec::new(),
        tasks: Vec::new(),
        guest_list: Vec::new(),
        registry: Vec::new(),
        status: "planning".to_string(),
    };

    WEDDING_STORAGE.with(|weddings| {
        weddings.borrow_mut().insert(wedding_id, wedding.clone());
    });

    Ok((
        wedding,
        Message::Success("Wedding created successfully".to_string()),
    ))
}

/**
 * Wedding Queries
 */

#[ic_cdk::query]
fn get_wedding_details(wedding_id: u64) -> Result<Wedding, Message> {
    WEDDING_STORAGE.with(|storage| {
        storage
            .borrow()
            .get(&wedding_id)
            .ok_or(Message::WeddingNotFound("Wedding not found".to_string()))
    })
}

// Get All Weddings
#[ic_cdk::query]
fn get_all_weddings() -> Result<Vec<Wedding>, Message> {
    WEDDING_STORAGE.with(|weddings| {
        let all_weddings: Vec<Wedding> = weddings
            .borrow()
            .iter()
            .map(|(_, wedding)| wedding.clone()) // Clone each wedding to return owned data
            .collect();

        if all_weddings.is_empty() {
            Err(Message::WeddingNotFound("No weddings found.".to_string()))
        } else {
            Ok(all_weddings)
        }
    })
}

// Get Wedding Timeline
#[ic_cdk::query]
fn get_wedding_timeline(wedding_id: u64) -> Result<Vec<TimelineItem>, Message> {
    WEDDING_STORAGE.with(|storage| {
        let wedding = storage.borrow().get(&wedding_id);

        match wedding {
            Some(wedding) => {
                if wedding.timeline.is_empty() {
                    Err(Message::NoTimeLineItemsFound(
                        "No timeline items found for this wedding".to_string(),
                    ))
                } else {
                    Ok(wedding.timeline.clone())
                }
            }
            None => Err(Message::WeddingNotFound(
                "Wedding with the provided ID not found".to_string(),
            )),
        }
    })
}

/*
 * Guest Management
 * RSVP Submission
 * Guest RSVP Approval
 * Table Assignment
 * Guest Queries
 * Guest RSVP Queries
 * Guest RSVP List
 * Guest RSVP Status
 * Guest RSVP Count
 */

// Guest RSVP Submission
#[ic_cdk::update]
fn guest_rsvp(payload: GuestRsvpPayload) -> Result<(String, Guest, Wedding), Message> {
    // Fetch the wedding from storage
    let wedding = WEDDING_STORAGE.with(|storage| storage.borrow().get(&payload.wedding_id));

    // Validate wedding existence
    let wedding = match wedding {
        Some(wedding) => wedding.clone(),
        None => {
            return Err(Message::WeddingNotFound(format!(
                "Wedding with ID {} not found",
                payload.wedding_id
            )))
        }
    };

    // Check if the guest already exists
    let guest_exists = wedding
        .guest_list
        .iter()
        .any(|guest| guest.guest_email == payload.guest_email);

    if guest_exists {
        return Err(Message::Error("Guest RSVP already submitted".to_string()));
    }

    // Create the new guest RSVP
    let guest = Guest {
        name: payload.name.clone(),
        guest_email: payload.guest_email.clone(),
        rsvp_status: "pending".to_string(),
        dietary_restrictions: payload.dietary_restrictions.clone(),
        plus_one: payload.plus_one,
        table_assignment: TableAssignment::Unassigned,
    };

    // Update the wedding's guest list
    let mut updated_wedding = wedding.clone();
    updated_wedding.guest_list.push(guest.clone());

    // Save the updated wedding details
    WEDDING_STORAGE.with(|storage| {
        storage
            .borrow_mut()
            .insert(payload.wedding_id, updated_wedding.clone());
    });

    // Return success
    Ok((
        "Guest RSVP submitted successfully".to_string(),
        guest,
        updated_wedding,
    ))
}

// Guest RSVP Approval and Table Assignment
#[ic_cdk::update]
fn approve_rsvp(payload: ApproveRsvpPayload) -> Result<(String, Guest, Wedding), Message> {
    // Use WEDDING_STORAGE to retrieve the wedding data
    let mut wedding = WEDDING_STORAGE.with(|storage| {
        storage
            .borrow()
            .get(&payload.wedding_id)
            .map(|w| w.clone()) // Explicitly clone the Wedding object
            .ok_or_else(|| {
                Message::WeddingNotFound(format!(
                    "Wedding with ID {} not found",
                    payload.wedding_id
                ))
            })
    })?;

    // Calculate confirmed guests and check seating capacity
    let confirmed_guests = wedding
        .guest_list
        .iter()
        .filter(|guest| guest.rsvp_status == "confirmed")
        .fold(0, |count, guest| count + if guest.plus_one { 2 } else { 1 });

    if confirmed_guests >= wedding.guest_count as usize {
        return Err(Message::BudgetExceeded(
            "Available seats exceeded the wedding limit.".to_string(),
        ));
    }

    // Find the guest in the guest list
    let guest_index = wedding
        .guest_list
        .iter()
        .position(|guest| guest.guest_email == payload.guest_email);

    let guest_index = match guest_index {
        Some(index) => index,
        None => {
            return Err(Message::Error(
                "Guest not found in the RSVP list.".to_string(),
            ));
        }
    };

    // Update guest details
    wedding.guest_list[guest_index].rsvp_status = "confirmed".to_string();
    wedding.guest_list[guest_index].table_assignment = payload.table_assignment.clone();

    // Save the updated wedding data back to storage
    WEDDING_STORAGE.with(|storage| {
        storage
            .borrow_mut()
            .insert(payload.wedding_id, wedding.clone());
    });

    // Return success message along with updated guest and wedding
    Ok((
        "RSVP approved successfully".to_string(),
        wedding.guest_list[guest_index].clone(),
        wedding,
    ))
}

// New Functions 

#[ic_cdk::update]
fn update_vendor_availability(vendor_id: u64, new_availability: Vec<String>) -> Result<Message, Message> {
    VENDOR_STORAGE.with(|storage| {
        let mut vendors = storage.borrow_mut();
        let vendor = vendors.get(&vendor_id);

        let mut vendor = vendor.ok_or_else(|| {
            Message::VendorNotFound(format!(
                "Vendor with ID {} not found",
                vendor_id
            ))
        })?;

        vendor.availability = new_availability;

        vendors.insert(vendor_id, vendor);

        Ok(Message::Success(
            "Vendor availability updated successfully".to_string(),
        ))
    })
}

// Search By wedding Date

#[ic_cdk::query]
fn search_weddings_by_date(date: String) -> Result<Vec<Wedding>, Message> {
    WEDDING_STORAGE.with(|storage| {
        let weddings: Vec<Wedding> = storage
            .borrow()
            .iter()
            .filter_map(|(_, wedding)| {
                if wedding.date == date {
                    Some(wedding.clone())
                } else {
                    None
                }
            })
            .collect();

        if weddings.is_empty() {
            Err(Message::WeddingNotFound(format!(
                "No weddings found on date: {}",
                date
            )))
        } else {
            Ok(weddings)
        }
    })
}


// Cancel Wedding Boking
#[ic_cdk::update]
fn cancel_vendor_booking(wedding_id: u64, vendor_id: u64) -> Result<Message, Message> {
    WEDDING_STORAGE.with(|storage| {
        let mut weddings = storage.borrow_mut();
        let wedding = weddings.get(&wedding_id);

        let mut wedding = wedding.ok_or_else(|| {
            Message::WeddingNotFound(format!(
                "Wedding with ID {} not found",
                wedding_id
            ))
        })?;

        let original_count = wedding.vendors.len();
        wedding.vendors.retain(|booking| booking.vendor_id != vendor_id);

        if wedding.vendors.len() == original_count {
            return Err(Message::VendorNotFound(format!(
                "Vendor with ID {} not booked for this wedding",
                vendor_id
            )));
        }

        weddings.insert(wedding_id, wedding);

        Ok(Message::Success(
            "Vendor booking canceled successfully".to_string(),
        ))
    })
}

// Mark Timeline Item as Completed
#[ic_cdk::update]
fn mark_timeline_item_completed(wedding_id: u64, time: String) -> Result<Message, Message> {
    WEDDING_STORAGE.with(|storage| {
        let mut weddings = storage.borrow_mut();
        let wedding = weddings.get(&wedding_id);

        let mut wedding = wedding.ok_or_else(|| {
            Message::WeddingNotFound(format!(
                "Wedding with ID {} not found",
                wedding_id
            ))
        })?;

        let mut updated = false;
        for item in wedding.timeline.iter_mut() {
            if item.time == time {
                item.status = "completed".to_string();
                updated = true;
            }
        }

        if !updated {
            return Err(Message::Error(format!(
                "No timeline item found with time: {}",
                time
            )));
        }

        weddings.insert(wedding_id, wedding);

        Ok(Message::Success(
            "Timeline item marked as completed successfully".to_string(),
        ))
    })
}


/*
 * Guest Queries
 */

// Guest List
#[ic_cdk::query]
fn get_guest_list(wedding_id: u64) -> Result<Vec<Guest>, Message> {
    WEDDING_STORAGE.with(|storage| {
        let wedding = storage.borrow().get(&wedding_id);

        match wedding {
            Some(wedding) => {
                if wedding.guest_list.is_empty() {
                    Err(Message::Error(
                        "No guests found for this wedding".to_string(),
                    ))
                } else {
                    Ok(wedding.guest_list.clone())
                }
            }
            None => Err(Message::WeddingNotFound(
                "Wedding with the provided ID not found".to_string(),
            )),
        }
    })
}

// Guest RSVP List
#[ic_cdk::query]
fn get_guest_details(wedding_id: u64, guest_email: String) -> Result<Guest, Message> {
    WEDDING_STORAGE.with(|storage| {
        let wedding = storage.borrow().get(&wedding_id);

        match wedding {
            Some(wedding) => {
                let guest = wedding
                    .guest_list
                    .iter()
                    .find(|guest| guest.guest_email == guest_email);

                match guest {
                    Some(guest) => Ok(guest.clone()),
                    None => Err(Message::Error("Guest not found".to_string())),
                }
            }
            None => Err(Message::WeddingNotFound(
                "Wedding with the provided ID not found".to_string(),
            )),
        }
    })
}

// Guest RSVP Status
#[ic_cdk::query]
fn get_guest_rsvp_status(wedding_id: u64, guest_email: String) -> Result<String, Message> {
    WEDDING_STORAGE.with(|storage| {
        let wedding = storage.borrow().get(&wedding_id);

        match wedding {
            Some(wedding) => {
                let guest = wedding
                    .guest_list
                    .iter()
                    .find(|guest| guest.guest_email == guest_email);

                match guest {
                    Some(guest) => Ok(guest.rsvp_status.clone()),
                    None => Err(Message::Error("Guest not found".to_string())),
                }
            }
            None => Err(Message::WeddingNotFound(
                "Wedding with the provided ID not found".to_string(),
            )),
        }
    })
}

// Guest RSVP Count
#[ic_cdk::query]
fn get_guest_rsvp_count(wedding_id: u64) -> Result<u64, Message> {
    WEDDING_STORAGE.with(|storage| {
        let wedding = storage.borrow().get(&wedding_id);

        match wedding {
            Some(wedding) => Ok(wedding.guest_list.len() as u64),
            None => Err(Message::WeddingNotFound(
                "Wedding with the provided ID not found".to_string(),
            )),
        }
    })
}

/*
 * Timeline Item Management
 */

// Add Timeline Item
#[ic_cdk::update]
fn add_timeline_item(
    payload: TimelineItemPayload,
) -> Result<(String, TimelineItem, Wedding), Message> {
    // Fetch the wedding from storage
    let wedding = WEDDING_STORAGE.with(|storage| storage.borrow().get(&payload.wedding_id));

    // Validate wedding existence
    let wedding = match wedding {
        Some(wedding) => wedding.clone(),
        None => {
            return Err(Message::WeddingNotFound(format!(
                "Wedding with ID {} not found",
                payload.wedding_id
            )))
        }
    };

    // Create the new timeline item
    let timeline_item = TimelineItem {
        wedding_id: payload.wedding_id,
        time: payload.time.clone(),
        description: payload.description.clone(),
        responsible: payload.responsible.clone(),
        status: payload.status.clone(),
    };

    // Update the wedding's timeline
    let mut updated_wedding = wedding.clone();
    updated_wedding.timeline.push(timeline_item.clone());

    // Save the updated wedding details
    WEDDING_STORAGE.with(|storage| {
        storage
            .borrow_mut()
            .insert(payload.wedding_id, updated_wedding.clone());
    });

    // Return success
    Ok((
        "Timeline item added successfully".to_string(),
        timeline_item,
        updated_wedding,
    ))
}

/**
 * Task Management
 */

// Add Task
#[ic_cdk::update]
fn add_task(payload: TaskPayload) -> Result<(String, Task, Wedding), Message> {
    // Fetch the wedding from storage
    let wedding = WEDDING_STORAGE.with(|storage| storage.borrow().get(&payload.wedding_id));

    // Validate wedding existence
    let wedding = match wedding {
        Some(wedding) => wedding.clone(),
        None => {
            return Err(Message::WeddingNotFound(format!(
                "Wedding with ID {} not found",
                payload.wedding_id
            )))
        }
    };

    // Generate a unique ID for the task
    let task_id = generate_uuid();

    // Create the new task
    let task = Task {
        id: task_id,
        title: payload.title.clone(),
        description: payload.description.clone(),
        deadline: payload.deadline.clone(),
        assigned_to: payload.assigned_to.clone(),
        status: "pending".to_string(),
        budget: payload.budget,
    };

    // Update the wedding's tasks
    let mut updated_wedding = wedding.clone();
    updated_wedding.tasks.push(task.clone());

    // Save the updated wedding details
    WEDDING_STORAGE.with(|storage| {
        storage
            .borrow_mut()
            .insert(payload.wedding_id, updated_wedding.clone());
    });

    // Return success
    Ok(("Task added successfully".to_string(), task, updated_wedding))
}

// Update Task Status
#[ic_cdk::update]
fn update_task_status(
    payload: UpdateTaskStatusPayload,
) -> Result<(String, Task, Wedding), Message> {
    // Fetch the wedding from storage
    let wedding = WEDDING_STORAGE.with(|storage| storage.borrow().get(&payload.wedding_id));

    // Validate wedding existence
    let wedding = match wedding {
        Some(wedding) => wedding.clone(),
        None => {
            return Err(Message::WeddingNotFound(format!(
                "Wedding with ID {} not found",
                payload.wedding_id
            )))
        }
    };

    // Fetch the task from the wedding
    let task = wedding.tasks.iter().find(|task| task.id == payload.task_id);

    let task = match task {
        Some(task) => task.clone(),
        None => {
            return Err(Message::Error("Task not found".to_string()));
        }
    };

    // Update the task status
    let mut updated_task = task.clone();
    updated_task.status = payload.status.clone();

    // Update the wedding's tasks
    let mut updated_wedding = wedding.clone();
    updated_wedding.tasks = updated_wedding
        .tasks
        .iter()
        .map(|task| {
            if task.id == payload.task_id {
                updated_task.clone()
            } else {
                task.clone()
            }
        })
        .collect();

    // Save the updated wedding details
    WEDDING_STORAGE.with(|storage| {
        storage
            .borrow_mut()
            .insert(payload.wedding_id, updated_wedding.clone());
    });

    // Return success
    Ok((
        "Task status updated successfully".to_string(),
        updated_task,
        updated_wedding,
    ))
}

// Delete Task
#[ic_cdk::update]
fn delete_task(payload: DeleteTaskPayload) -> Result<(String, Task, Wedding), Message> {
    // Fetch the wedding from storage
    let wedding = WEDDING_STORAGE.with(|storage| storage.borrow().get(&payload.wedding_id));

    // Validate wedding existence
    let wedding = match wedding {
        Some(wedding) => wedding.clone(),
        None => {
            return Err(Message::WeddingNotFound(format!(
                "Wedding with ID {} not found",
                payload.wedding_id
            )))
        }
    };

    // Fetch the task from the wedding
    let task = wedding.tasks.iter().find(|task| task.id == payload.task_id);

    let task = match task {
        Some(task) => task.clone(),
        None => {
            return Err(Message::Error("Task not found".to_string()));
        }
    };

    // Remove the task from the wedding's tasks
    let mut updated_wedding = wedding.clone();
    let updated_tasks: Vec<Task> = updated_wedding
        .tasks
        .iter()
        .filter(|task| task.id != payload.task_id)
        .cloned()
        .collect();

    updated_wedding.tasks = updated_tasks;

    // Save the updated wedding details
    WEDDING_STORAGE.with(|storage| {
        storage
            .borrow_mut()
            .insert(payload.wedding_id, updated_wedding.clone());
    });

    // Return success
    Ok((
        "Task deleted successfully".to_string(),
        task,
        updated_wedding,
    ))
}

// Get Task List
#[ic_cdk::query]
fn get_task_list(wedding_id: u64) -> Result<Vec<Task>, Message> {
    WEDDING_STORAGE.with(|storage| {
        let wedding = storage.borrow().get(&wedding_id);

        match wedding {
            Some(wedding) => {
                if wedding.tasks.is_empty() {
                    Err(Message::Error(
                        "No tasks found for this wedding".to_string(),
                    ))
                } else {
                    Ok(wedding.tasks.clone())
                }
            }
            None => Err(Message::WeddingNotFound(
                "Wedding with the provided ID not found".to_string(),
            )),
        }
    })
}

// Get Task Details
#[ic_cdk::query]
fn get_task_details(wedding_id: u64, task_id: u64) -> Result<Task, Message> {
    WEDDING_STORAGE.with(|storage| {
        let wedding = storage.borrow().get(&wedding_id);

        match wedding {
            Some(wedding) => {
                let task = wedding.tasks.iter().find(|task| task.id == task_id);

                match task {
                    Some(task) => Ok(task.clone()),
                    None => Err(Message::Error("Task not found".to_string())),
                }
            }
            None => Err(Message::WeddingNotFound(
                "Wedding with the provided ID not found".to_string(),
            )),
        }
    })
}

/*
 * Registry Management
 */

// Add Registry Item
#[ic_cdk::update]
fn add_registry_item(
    payload: AddRegistryItemPayload,
) -> Result<(String, RegistryItem, Wedding), Message> {
    // Fetch the wedding from storage
    let wedding = WEDDING_STORAGE.with(|storage| storage.borrow().get(&payload.wedding_id));

    // Validate wedding existence
    let wedding = match wedding {
        Some(wedding) => wedding.clone(),
        None => {
            return Err(Message::WeddingNotFound(format!(
                "Wedding with ID {} not found",
                payload.wedding_id
            )))
        }
    };

    // Check if the item already exists
    let item_exists = wedding
        .registry
        .iter()
        .any(|item| item.name == payload.name);

    if item_exists {
        return Err(Message::Error("Registry item already exists".to_string()));
    }

    // Create the new registry item
    let registry_item = RegistryItem {
        name: payload.name.clone(),
        description: payload.description.clone(),
        price: payload.price,
        status: "available".to_string(),
        purchased_by: "".to_string(),
    };

    // Update the wedding's registry
    let mut updated_wedding = wedding.clone();
    updated_wedding.registry.push(registry_item.clone());

    // Save the updated wedding details
    WEDDING_STORAGE.with(|storage| {
        storage
            .borrow_mut()
            .insert(payload.wedding_id, updated_wedding.clone());
    });

    // Return success
    Ok((
        "Registry item added successfully".to_string(),
        registry_item,
        updated_wedding,
    ))
}

// Update Registry Item Status
#[ic_cdk::update]
fn update_registry_item_status(
    payload: UpdateRegistryItemStatusPayload,
) -> Result<(String, RegistryItem, Wedding), Message> {
    // Fetch the wedding from storage
    let wedding = WEDDING_STORAGE.with(|storage| storage.borrow().get(&payload.wedding_id));

    // Validate wedding existence
    let wedding = match wedding {
        Some(wedding) => wedding.clone(),
        None => {
            return Err(Message::WeddingNotFound(format!(
                "Wedding with ID {} not found",
                payload.wedding_id
            )))
        }
    };

    // Fetch the registry item from the wedding
    let item = wedding
        .registry
        .iter()
        .find(|item| item.name == payload.item_name);

    let item = match item {
        Some(item) => item.clone(),
        None => {
            return Err(Message::Error("Registry item not found".to_string()));
        }
    };

    // Update the registry item status
    let mut updated_item = item.clone();
    updated_item.status = payload.status.clone();
    updated_item.purchased_by = payload.purchased_by.clone();

    // Update the wedding's registry
    let mut updated_wedding = wedding.clone();
    updated_wedding.registry = updated_wedding
        .registry
        .iter()
        .map(|item| {
            if item.name == payload.item_name {
                updated_item.clone()
            } else {
                item.clone()
            }
        })
        .collect();

    // Save the updated wedding details
    WEDDING_STORAGE.with(|storage| {
        storage
            .borrow_mut()
            .insert(payload.wedding_id, updated_wedding.clone());
    });

    // Return success
    Ok((
        "Registry item status updated successfully".to_string(),
        updated_item,
        updated_wedding,
    ))
}

// Delete Registry Item
#[ic_cdk::update]
fn delete_registry_item(
    payload: DeleteRegistryItemPayload,
) -> Result<(String, RegistryItem, Wedding), Message> {
    // Fetch the wedding from storage
    let wedding = WEDDING_STORAGE.with(|storage| storage.borrow().get(&payload.wedding_id));

    // Validate wedding existence
    let wedding = match wedding {
        Some(wedding) => wedding.clone(),
        None => {
            return Err(Message::WeddingNotFound(format!(
                "Wedding with ID {} not found",
                payload.wedding_id
            )))
        }
    };

    // Fetch the registry item from the wedding
    let item = wedding
        .registry
        .iter()
        .find(|item| item.name == payload.item_name);

    let item = match item {
        Some(item) => item.clone(),
        None => {
            return Err(Message::Error("Registry item not found".to_string()));
        }
    };

    // Remove the item from the wedding's registry
    let mut updated_wedding = wedding.clone();
    let updated_registry: Vec<RegistryItem> = updated_wedding
        .registry
        .iter()
        .filter(|item| item.name != payload.item_name)
        .cloned()
        .collect();

    updated_wedding.registry = updated_registry;

    // Save the updated wedding details
    WEDDING_STORAGE.with(|storage| {
        storage
            .borrow_mut()
            .insert(payload.wedding_id, updated_wedding.clone());
    });

    // Return success
    Ok((
        "Registry item deleted successfully".to_string(),
        item,
        updated_wedding,
    ))
}

// Get Registry Items
#[ic_cdk::query]
fn get_registry_items(wedding_id: u64) -> Result<Vec<RegistryItem>, Message> {
    WEDDING_STORAGE.with(|storage| {
        let wedding = storage.borrow().get(&wedding_id);

        match wedding {
            Some(wedding) => {
                if wedding.registry.is_empty() {
                    Err(Message::Error(
                        "No registry items found for this wedding".to_string(),
                    ))
                } else {
                    Ok(wedding.registry.clone())
                }
            }
            None => Err(Message::WeddingNotFound(
                "Wedding with the provided ID not found".to_string(),
            )),
        }
    })
}

// Get Registry Item Details
#[ic_cdk::query]
fn get_registry_item_details(wedding_id: u64, item_name: String) -> Result<RegistryItem, Message> {
    WEDDING_STORAGE.with(|storage| {
        let wedding = storage.borrow().get(&wedding_id);

        match wedding {
            Some(wedding) => {
                let item = wedding.registry.iter().find(|item| item.name == item_name);

                match item {
                    Some(item) => Ok(item.clone()),
                    None => Err(Message::Error("Registry item not found".to_string())),
                }
            }
            None => Err(Message::WeddingNotFound(
                "Wedding with the provided ID not found".to_string(),
            )),
        }
    })
}

// Export Candid interface
ic_cdk::export_candid!();
