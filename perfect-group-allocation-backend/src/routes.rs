use zero_cost_templating::template_stream;

//pub mod favicon;
pub mod index;
pub mod indexcss;
//pub mod openid_login;
//pub mod openid_redirect;
pub mod projects;

#[template_stream("templates")]
pub fn temporary() {}
