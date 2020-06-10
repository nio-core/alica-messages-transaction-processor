use sha2::Digest;

pub struct Handler {
    family_name: String,
    family_versions: Vec<String>,
    family_namespaces: Vec<String>,
}

impl Handler {
    pub fn new() -> Self {
        let family_name = "alica-messages";
        let mut hasher = sha2::Sha512::new();
        hasher.input(family_name);
        let result = hasher.result();

        let namespace = data_encoding::HEXUPPER.encode(&result[..6]);

        Handler {
            family_name: String::from(family_name),
            family_versions: vec![String::from("0.1.0")],
            family_namespaces: vec![namespace],
        }
    }
}

impl sawtooth_sdk::processor::handler::TransactionHandler for Handler {
    fn family_name(&self) -> String {
        self.family_name.clone()
    }

    fn family_versions(&self) -> Vec<String> {
        self.family_versions.clone()
    }

    fn namespaces(&self) -> Vec<String> {
        self.family_namespaces.clone()
    }

    fn apply(
        &self,
        request: &sawtooth_sdk::messages::processor::TpProcessRequest,
        context: &mut dyn sawtooth_sdk::processor::handler::TransactionContext,
    ) -> Result<(), sawtooth_sdk::processor::handler::ApplyError> {
        println!(
            "Transaction received from {}!",
            &request.get_header().get_signer_public_key()[..6]
        );
        Ok(())
    }
}

#[cfg(test)]
mod test {}
