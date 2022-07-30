use hdk::prelude::holo_hash::*;
use hdk::prelude::*;

#[derive(Serialize, Deserialize, Debug, SerializedBytes)]
struct Properties {
    progenitor: AgentPubKey,
}

#[hdk_extern]
fn progenitor(_: ()) -> ExternResult<AgentPubKey> {
    let serialized = dna_info()?.properties;
    let props = Properties::try_from(serialized).map_err(|e| wasm_error!(e.into()))?;
    Ok(props.progenitor)
}

#[hdk_extern]
fn request_read_all_posts(_: ()) -> ExternResult<Vec<Record>> {
    let call_response = call(
        CallTargetCell::OtherRole("private_publication".to_string()),
        "posts".to_string(),
        FunctionName("get_all_posts".to_string()),
        None,
        (),
    )?;

    match call_response {
        ZomeCallResponse::Ok(ret) => ret.decode().map_err(|e| wasm_error!(e.into())),
        _ => Err(wasm_error!(WasmErrorInner::Guest(
            "cross-zome call failed".to_string()
        ))),
    }
}

#[hdk_extern]
fn grant_capability_to_read(pubkey: AgentPubKey) -> ExternResult<CapSecret> {
    // let mut byte_array: [u8; 64] = [0; 64];
    // byte_array.copy_from_slice(random_bytes(64)?.as_slice());
    // let secret = byte_array.into();

    let secret = CapSecret::try_from(random_bytes(64)?.into_vec())
        .map_err(|_| wasm_error!(WasmErrorInner::Guest("could not build secret".into())))?;
    let mut functions = BTreeSet::new();
    functions.insert((
        zome_info()?.name,
        FunctionName::from("request_read_all_posts"),
    ));
    let mut grantee = BTreeSet::new();
    grantee.insert(pubkey);
    let cap_grant_entry = CapGrantEntry {
        tag: String::from("bob"),
        access: CapAccess::Assigned {
            secret,
            assignees: grantee,
        },
        functions,
    };
    create_cap_grant(cap_grant_entry)?;

    Ok(secret)
}

#[hdk_extern]
fn store_capability_claim(secret: CapSecret) -> ExternResult<()> {
    let progenitor = progenitor(())?;
    let cap_claim_entry = CapClaimEntry {
        grantor: progenitor,
        secret,
        tag: "progenitor".into(),
    };
    create_cap_claim(cap_claim_entry)?;

    Ok(())
}

#[hdk_extern]
fn get_claim(_: ()) -> ExternResult<CapClaim> {
    let progenitor = progenitor(())?;
    let filter = ChainQueryFilter::new()
        .entry_type(EntryType::CapClaim)
        .include_entries(true);
    let claim_records = query(filter)?;
    let rec = claim_records.first().unwrap().clone();
    let entry = rec.entry;
    let claim = if let RecordEntry::Present(Entry::CapClaim(claim)) = entry {
        Some(claim)
    } else {
        None
    };
    Ok(claim.unwrap())
}

#[hdk_extern]
fn read_all_posts(_: ()) -> ExternResult<Vec<Record>> {
    // - Query the source chain to get the capability claim.
    let progenitor = progenitor(())?;
    let filter = ChainQueryFilter::new()
        .entry_type(EntryType::CapClaim)
        .include_entries(true);
    let claim_records = query(filter)?;
    let claim = claim_records
        .into_iter()
        .map(|rec| rec.entry().clone())
        .filter_map(|claim| {
            if let RecordEntry::Present(Entry::CapClaim(claim)) = claim {
                Some(claim)
            } else {
                None
            }
        })
        .find(|claim| claim.grantor == progenitor);
    // .last();

    // - Call remote to the progenitor agent's `request_read_all_posts`.
    if let Some(claim) = claim {
        let response = call_remote(
            progenitor,
            zome_info()?.name,
            FunctionName("request_read_all_posts".into()),
            Some(claim.secret),
            (),
        )?;
        match response {
            ZomeCallResponse::Ok(bytes) => {
                let records: Vec<Record> = decode(&bytes).map_err(|e| wasm_error!(e.into()))?;
                Ok(records)
            }
            ZomeCallResponse::Unauthorized(_, _, _, _) => Err(wasm_error!(WasmErrorInner::Guest(
                "Unauthorized! Claim did not work".into()
            ))),
            ZomeCallResponse::NetworkError(e) => Err(wasm_error!(WasmErrorInner::Guest(e))),
            _ => Err(wasm_error!(WasmErrorInner::Guest("Unknown error".into()))),
        }
    } else {
        Err(wasm_error!(WasmErrorInner::Guest("No claim found".into())))
    }
    // - Return the result.
}

/** Don't change */
#[cfg(feature = "exercise")]
extern crate private_publication_lobby;
