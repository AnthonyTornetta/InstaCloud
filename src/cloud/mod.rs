use api::ApiEndpoint;

pub mod api;

#[derive(Default)]
pub struct Cloud {
    apis: Vec<ApiEndpoint>,
}

impl Cloud {
    pub fn set_apis(&mut self, apis: Vec<ApiEndpoint>) {
        self.apis = apis;
    }
}
