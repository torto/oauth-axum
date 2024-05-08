use crate::CustomProvider;

pub struct PaypalProvider {}

impl PaypalProvider {
    pub fn new(client_id: String, client_secret: String, redirect_url: String) -> CustomProvider {
        CustomProvider::new(
            String::from("https://sandbox.paypal.com/signin/authorize"),
            String::from("https://api-m.sandbox.paypal.com/v1/oauth2/token"),
            client_id,
            client_secret,
            redirect_url,
        )
    }
}
