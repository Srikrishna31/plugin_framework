// use crate::plugin::Plugin;
//
// /// A plugin which allows web based plugins to process request and responses
// pub trait RequestResponsePlugin : Plugin {
//     /// Inspect (and possibly mutate) the request before it is sent.
//     fn pre_send(&self, _request: &mut Request) {}
//
//     /// Inspect and/or mutate the received response before it is displayed to the user.
//     fn post_receive(&self, _response: &mut Response) {}
// }
