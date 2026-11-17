//! Credentials stored in the current user's system keyring.
//!
//! macOS uses Keychain, Windows uses Credential Manager, and other Unix
//! platforms use Secret Service. There is no plaintext fallback. The keyring
//! is opened only when a credential is constructed, not during chat startup.
//!
//! ```no_run
//! use rh::credentials::Credential;
//!
//! # fn example(api_key: &str) -> keyring::Result<()> {
//! let credential = Credential::new("openai/api-key")?;
//! credential.set(api_key)?;
//! let saved_key = credential.get()?;
//! // Pass saved_key to the provider without logging it.
//! credential.delete()?;
//! # Ok(())
//! # }
//! ```
//!
//! Returned passwords are ordinary strings. Callers must not log them or
//! persist them in configuration files. Native error objects may also contain
//! sensitive data; avoid logging their Debug representation.

const SERVICE: &str = "rh";

/// One named credential under the `rh` keyring service.
///
/// This wrapper deliberately does not implement Debug to avoid exposing secrets.
pub struct Credential {
    entry: keyring::Entry,
}

impl Credential {
    /// Open a credential identified by a stable account name, such as `openai/api-key`.
    ///
    /// Blank account names are rejected. Other names are passed through unchanged.
    /// Native keyring initialization or access errors are returned to the caller.
    pub fn new(account: &str) -> keyring::Result<Self> {
        if account.trim().is_empty() {
            return Err(keyring::Error::Invalid(
                "account".into(),
                "must not be blank".into(),
            ));
        }
        Ok(Self {
            entry: keyring::Entry::new(SERVICE, account)?,
        })
    }

    /// Store or replace the credential. The value is never written to rh's files.
    pub fn set(&self, secret: &str) -> keyring::Result<()> {
        self.entry.set_password(secret)
    }

    /// Read the credential, or return None if it has not been stored.
    ///
    /// Locked, unavailable, or corrupt storage is an error, not a missing value.
    pub fn get(&self) -> keyring::Result<Option<String>> {
        match self.entry.get_password() {
            Ok(secret) => Ok(Some(secret)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(error) => Err(error),
        }
    }

    /// Delete the credential. Deleting a missing credential also succeeds.
    pub fn delete(&self) -> keyring::Result<()> {
        match self.entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use keyring_core::{api::CredentialStoreApi, mock};

    fn credential(store: &mock::Store, account: &str) -> Credential {
        Credential {
            entry: keyring::Entry {
                inner: store.build(SERVICE, account, None).unwrap(),
            },
        }
    }

    #[test]
    fn missing_credentials_are_optional_and_deletion_is_idempotent() {
        let store = mock::Store::new().unwrap();
        let credential = credential(&store, "openai/api-key");
        assert_eq!(credential.get().unwrap(), None);
        credential.delete().unwrap();
    }

    #[test]
    fn stores_retrieves_replaces_and_deletes_credentials() {
        let store = mock::Store::new().unwrap();
        let first = credential(&store, "openai/api-key");
        let reopened = credential(&store, "openai/api-key");
        first.set("test-key-one").unwrap();
        assert_eq!(reopened.get().unwrap().as_deref(), Some("test-key-one"));
        reopened.set("test-key-two").unwrap();
        assert_eq!(first.get().unwrap().as_deref(), Some("test-key-two"));
        first.delete().unwrap();
        assert_eq!(reopened.get().unwrap(), None);
        reopened.delete().unwrap();
    }

    #[test]
    fn accounts_are_independent_and_use_the_rh_service() {
        let store = mock::Store::new().unwrap();
        let first = credential(&store, "openai/api-key");
        let second = credential(&store, "anthropic/api-key");
        first.set("first-key").unwrap();
        second.set("second-key").unwrap();
        first.delete().unwrap();
        assert_eq!(second.get().unwrap().as_deref(), Some("second-key"));
        let entry = store.build("rh", "anthropic/api-key", None).unwrap();
        assert_eq!(entry.get_password().unwrap(), "second-key");
    }

    #[test]
    fn rejects_blank_accounts_before_opening_the_native_keyring() {
        for account in ["", " ", "\n\t"] {
            assert!(matches!(
                Credential::new(account),
                Err(keyring::Error::Invalid(_, _))
            ));
        }
    }

    #[test]
    fn propagates_storage_errors_for_each_operation() {
        let store = mock::Store::new().unwrap();
        let credential = credential(&store, "openai/api-key");
        let mock = credential
            .entry
            .inner
            .as_any()
            .downcast_ref::<mock::Cred>()
            .unwrap();
        let locked = || keyring::Error::NoStorageAccess(std::io::Error::other("locked").into());
        mock.set_error(locked());
        assert!(matches!(
            credential.set("test-key"),
            Err(keyring::Error::NoStorageAccess(_))
        ));
        mock.set_error(locked());
        assert!(matches!(
            credential.get(),
            Err(keyring::Error::NoStorageAccess(_))
        ));
        mock.set_error(locked());
        assert!(matches!(
            credential.delete(),
            Err(keyring::Error::NoStorageAccess(_))
        ));
    }
}
