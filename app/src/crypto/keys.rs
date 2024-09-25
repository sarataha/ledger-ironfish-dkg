/*****************************************************************************
 *   Ledger App Ironfish Rust.
 *   (c) 2023 Ledger SAS.
 *
 *  Licensed under the Apache License, Version 2.0 (the "License");
 *  you may not use this file except in compliance with the License.
 *  You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 *  Unless required by applicable law or agreed to in writing, software
 *  distributed under the License is distributed on an "AS IS" BASIS,
 *  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *  See the License for the specific language governing permissions and
 *  limitations under the License.
 *****************************************************************************/

use crate::bolos::zlog_stack;
use crate::ironfish::multisig::{derive_account_keys, MultisigAccountKeys};
#[cfg(feature = "ledger")]
use crate::nvm::dkg_keys::DkgKeys;
use crate::AppSW;
use alloc::vec::Vec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstantKey {
    SpendingKeyGenerator,
    ProofGenerationKeyGenerator,
    PublicKeyGenerator,
}

#[cfg(feature = "ledger")]
pub(crate) fn get_dkg_keys() -> Result<MultisigAccountKeys, AppSW> {
    zlog_stack("start handler_dkg_get_keys\0");

    let group_secret_key = DkgKeys.load_group_secret_key()?;
    let frost_public_key_package = DkgKeys.load_frost_public_key_package()?;

    let verifying_key: [u8; 32] = frost_public_key_package
        .verifying_key()
        .serialize()
        .map_err(|_| AppSW::InvalidKeyType)?
        .as_slice()
        .try_into()
        .map_err(|_| AppSW::InvalidKeyType)?;

    Ok(derive_account_keys(&verifying_key, &group_secret_key))
}

#[inline(never)]
pub(crate) fn multisig_to_key_type(
    account_keys: &MultisigAccountKeys,
    key_type: u8,
) -> Result<Vec<u8>, AppSW> {
    zlog_stack("start get_requested_keys\0");

    let mut resp: Vec<u8> = Vec::with_capacity(32 * 4);
    match key_type {
        0 => {
            let data = account_keys.public_address.public_address();
            resp.extend_from_slice(&data);

            Ok(resp)
        }
        1 => {
            resp.extend_from_slice(account_keys.view_key.authorizing_key.to_bytes().as_ref());
            resp.extend_from_slice(
                account_keys
                    .view_key
                    .nullifier_deriving_key
                    .to_bytes()
                    .as_ref(),
            );
            resp.extend_from_slice(account_keys.incoming_viewing_key.view_key.as_ref());
            resp.extend_from_slice(account_keys.outgoing_viewing_key.view_key.as_ref());
            Ok(resp)
        }
        2 => {
            resp.extend_from_slice(account_keys.view_key.authorizing_key.to_bytes().as_ref());
            resp.extend_from_slice(account_keys.proof_authorizing_key.to_bytes().as_ref());
            Ok(resp)
        }
        _ => Err(AppSW::InvalidKeyType),
    }
}
