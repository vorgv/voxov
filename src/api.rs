use crate::config::Config;
use crate::auth::Auth;

pub struct Api {
    auth: Auth
}

impl Api {
    pub fn new(_config: &Config, auth: Auth) -> Api {
        Api {
            auth,
        }
    }
    /// Open end points
    pub fn serve(&self) {
        self.serve_static();
        //TODO serve metadata
        //TODO serve config
    }
    /// Serve static big files
    fn serve_static(&self) {
        println!("hey");
        // SET file
        // GET file
    }
}
