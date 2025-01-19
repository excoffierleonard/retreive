use actix_web::web::ServiceConfig;

mod input_data;
//mod output_data;

// Function to configure all routes
pub fn v1_routes(cfg: &mut ServiceConfig) {
    cfg.service(input_data::input);
    // .service(output_data::login_user);
}
