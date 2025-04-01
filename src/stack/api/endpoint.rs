use crate::stack::lambda::LambdaFunction;

pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
}

pub struct ApiEndpoint {
    pub lambda: LambdaFunction,
    pub http_method: HttpMethod,
}
