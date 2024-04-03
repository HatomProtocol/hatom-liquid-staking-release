#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait AdminModule {
    /// Sets the admin address to the given address.
    ///
    /// # Arguments:
    ///
    /// - `admin` - The new admin address.
    ///
    fn set_admin_internal(&self, admin: &ManagedAddress) {
        require!(!admin.is_zero(), "cannot be address zero");

        self.admin().set(admin);
        self.new_admin_event(admin);
    }

    /// Checks if the caller is the current admin. Otherwise, it panics with an error message.
    ///
    fn require_admin(&self) {
        let admin = self.get_admin();
        let caller = self.blockchain().get_caller();
        require!(caller == admin, "caller must be admin");
    }

    /// Sets the admin address only if it has not already been set. If it has already been set, this function does
    /// nothing (otherwise, upgrades could change it).
    ///
    /// # Arguments:
    ///
    /// - `opt_admin` - An optional value of the new admin address.
    ///
    /// # Notes:
    ///
    /// - If `opt_admin` is `OptionalValue::None`, the caller of the function is set as the admin.
    ///
    fn try_set_admin(&self, opt_admin: OptionalValue<ManagedAddress>) {
        if self.admin().is_empty() {
            // default admin is caller
            let admin = match opt_admin {
                OptionalValue::None => self.blockchain().get_caller(),
                OptionalValue::Some(admin) => admin,
            };
            self.set_admin_internal(&admin);
        }
    }

    /// Returns the current admin address.
    ///
    /// # Returns:
    ///
    /// - The current admin address.
    ///
    #[view(getAdmin)]
    fn get_admin(&self) -> ManagedAddress {
        self.admin().get()
    }

    /// Returns the current pending admin address, if there is one.
    ///
    /// # Returns:
    ///
    /// - An `Option` containing the pending admin address if there is one, or `None` if there is not.
    ///
    #[view(getPendingAdmin)]
    fn get_pending_admin(&self) -> Option<ManagedAddress> {
        if self.pending_admin().is_empty() {
            None
        } else {
            let pending_admin = self.pending_admin().get();
            Some(pending_admin)
        }
    }

    /// Sets the pending admin address to the given address.
    ///
    /// # Arguments:
    ///
    /// - `new_pending_admin` - The new pending admin address.
    ///
    #[endpoint(setPendingAdmin)]
    fn set_pending_admin(&self, pending_admin: &ManagedAddress) {
        self.require_admin();

        require!(!pending_admin.is_zero(), "cannot be address zero");

        self.pending_admin().set(pending_admin);
        self.new_pending_admin_event(pending_admin);
    }

    /// Attempts to accept the pending admin, which must be set first using the `set_pending_admin` endpoint.
    #[endpoint(acceptAdmin)]
    fn accept_admin(&self) {
        let pending_admin = self.get_pending_admin();

        match pending_admin {
            None => sc_panic!("missing pending admin"),
            Some(new_admin) => {
                let caller = self.blockchain().get_caller();
                require!(caller == new_admin, "unauthorized, only pending admin can accept");

                self.pending_admin().clear();
                self.set_admin_internal(&new_admin);
            },
        }
    }

    /// Stores the admin address
    #[storage_mapper("admin")]
    fn admin(&self) -> SingleValueMapper<ManagedAddress>;

    /// Stores the pending admin address
    #[storage_mapper("pending_admin")]
    fn pending_admin(&self) -> SingleValueMapper<ManagedAddress>;

    /// Event emitted when the pending admin is updated.
    #[event("new_pending_admin_event")]
    fn new_pending_admin_event(&self, #[indexed] pending_admin: &ManagedAddress);

    /// Event emitted when the admin is updated.
    #[event("new_admin_event")]
    fn new_admin_event(&self, #[indexed] admin: &ManagedAddress);
}
