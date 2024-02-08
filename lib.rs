#![cfg_attr(not(feature = "std"), no_std, no_main)]

// SC updated with ❤️ by RpGmAx

#[ink::contract]
mod ronin_mission5_user {
    use ink::storage::Mapping;
    use ink_prelude::string::String;
    use ink_prelude::vec::Vec;
    use scale::{Decode, Encode};

    #[derive(Debug, PartialEq, Eq, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    // Enum for all the CRUD errors, nice way to handle them
    pub enum CrudError {
        YouAlreadyCreatedAMessage,
        SenderNotFound,
        YourMessageIsEmpty,
        YourMessageIsTooShort,
        NoMessageYet,
        YourMessageIsTheSameAsBefore,
        OwnerOnly,
    }

    #[derive(Debug, PartialEq, Eq, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    // Say Hi to the new Messages structure, used for read only ;)
    pub struct Messages {
        sender: AccountId,
        message: String,
    }

    #[derive(Debug, PartialEq, Eq, Encode, Decode, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    // And two more structures for update/delete history.
    pub struct UpdateHistory {
        sender: AccountId,
        old_message: String,
        new_message: String,
        timestamp: Timestamp,
    }

    #[derive(Debug, PartialEq, Eq, Encode, Decode, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct DeleteHistory {
        sender: AccountId,
        message: String,
        timestamp: Timestamp,
    }

    // Events are good, especially for dApps
    #[ink(event)]
    pub struct MessageCreated {
        #[ink(topic)]
        sender: AccountId,
        message: String,
    }

    #[ink(event)]
    pub struct MessageUpdated {
        #[ink(topic)]
        sender: AccountId,
        new_message: String,
    }

    #[ink(event)]
    pub struct MessageDeleted {
        #[ink(topic)]
        sender: AccountId,
    }

    #[ink(storage)]
    // Structure for both messages and senders storage and some new things
    pub struct CrudContract {
        messages: Mapping<AccountId, String>,
        senders: Vec<AccountId>,
        updates: Vec<UpdateHistory>,
        deletions: Vec<DeleteHistory>,
        owner: AccountId,
    }

    // Let's implement the CrudContract with a default message for the deployer (in the constructor) + updates/deletion and owner
    impl CrudContract {
        #[ink(constructor)]
        pub fn new() -> Self {
            let creator = Self::env().caller();

            let mut messages = Mapping::new();
            let init_message = String::from("I created my CRUD contract");
            messages.insert(creator, &init_message);

            let mut senders = Vec::new();
            senders.push(creator);

            let updates = Vec::new();
            let deletions = Vec::new();

            let owner = creator;

            Self {
                messages,
                senders,
                updates,
                deletions,
                owner,
            }
        }

        #[ink(message)]
        // Public function to create a new message (C in CRUD) - Updated with 2 verifications
        pub fn create_message(&mut self, message: String) -> Result<(), CrudError> {
            let caller = self.env().caller();

            if self.messages.contains(caller) {
                return Err(CrudError::YouAlreadyCreatedAMessage);
            }

            // Simple length verifications & new custom CRUD error
            if message.len() == 0 {
                return Err(CrudError::YourMessageIsEmpty);
            }
            if message.len() < 10 {
                return Err(CrudError::YourMessageIsTooShort);
            }

            self.messages.insert(caller, &message);
            self.senders.push(caller);
            // Like events ;)
            self.env().emit_event(MessageCreated {
                sender: caller,
                message,
            });
            Ok(())
        }

        #[ink(message)]
        // Public function to get message sent by a specific sender (R in CRUD)
        pub fn read_message_from(&mut self, sender: AccountId) -> Result<String, CrudError> {
            // Alternative method to avoid if/else condition
            self.messages.get(&sender).ok_or(CrudError::SenderNotFound)
        }

        #[ink(message)]
        // Public function to get all messages. Now with a new definition based on the new Messages structure
        pub fn read_all_messages(&mut self) -> Result<Vec<Messages>, CrudError> {
            if self.senders.is_empty() {
                return Err(CrudError::NoMessageYet);
            }

            // New way to return messages, via the Messages structure
            let all_messages = self
                .senders
                .iter()
                .filter_map(|sender| {
                    self.messages.get(sender).map(|message| Messages {
                        sender: *sender,
                        message,
                    })
                })
                .collect::<Vec<_>>();

            Ok(all_messages)
        }

        #[ink(message)]
        // New public function to allow the user to update their own message, if exists (U in CRUD)
        pub fn update_message(&mut self, new_message: String) -> Result<(), CrudError> {
            let caller = self.env().caller();

            // Simply check if the user already sent a message or not
            let current_message = self
                .messages
                .get(&caller)
                .ok_or(CrudError::SenderNotFound)?;

            // We must check the length, again
            if new_message.len() == 0 {
                return Err(CrudError::YourMessageIsEmpty);
            }
            if new_message.len() < 10 {
                return Err(CrudError::YourMessageIsTooShort);
            }

            // Then we check if the new message is the same as the old one
            if current_message == new_message {
                return Err(CrudError::YourMessageIsTheSameAsBefore);
            }

            // Tracking !
            self.updates.push(UpdateHistory {
                sender: caller,
                old_message: current_message.clone(),
                new_message: new_message.clone(),
                timestamp: Self::env().block_timestamp(),
            });

            // If all's right : we can update !
            self.messages.insert(caller, &new_message);
            // Event, again ;)
            self.env().emit_event(MessageUpdated {
                sender: caller,
                new_message,
            });
            Ok(())
        }

        #[ink(message)]
        // New public function to allow the user to delete their own message, if exists (D in CRUD)
        pub fn delete_message(&mut self) -> Result<(), CrudError> {
            let caller = self.env().caller();

            // We must check the caller already sent a message
            let message = self
                .messages
                .get(&caller)
                .ok_or(CrudError::SenderNotFound)?;

            // Tracking !
            self.deletions.push(DeleteHistory {
                sender: caller,
                message: message.clone(),
                timestamp: Self::env().block_timestamp(),
            });

            // If all's right : we can delete the appropriate message
            self.messages.remove(&caller);
            // And we keep all senders except the caller
            self.senders.retain(|&x| x != caller);
            // Event, the latest !
            self.env().emit_event(MessageDeleted { sender: caller });

            Ok(())
        }

        #[ink(message)]
        // Owner function to get update history
        pub fn get_update_history(&self) -> Result<Vec<UpdateHistory>, CrudError> {
            if self.env().caller() != self.owner {
                return Err(CrudError::OwnerOnly);
            }
            Ok(self.updates.clone())
        }

        #[ink(message)]
        // Owner function to get delete history
        pub fn get_delete_history(&self) -> Result<Vec<DeleteHistory>, CrudError> {
            if self.env().caller() != self.owner {
                return Err(CrudError::OwnerOnly);
            }
            Ok(self.deletions.clone())
        }

        // We now have a real CRUD ;)
        // SC updated with ❤️ by RpGmAx
    }
}
