use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread::spawn,
    time::SystemTime,
};

#[derive(Clone)]
pub struct AxumState {
    db: Arc<Mutex<HashMap<String, ItemOauthAxum>>>,
}

#[derive(Clone, Debug)]
pub struct ItemOauthAxum {
    pub verifier: String,
    pub created_at: SystemTime,
}

impl AxumState {
    pub fn new() -> Self {
        let db: Arc<Mutex<HashMap<String, ItemOauthAxum>>> = Arc::new(Mutex::new(HashMap::new()));
        let db_binding = Arc::clone(&db);
        spawn(move || loop {
            std::thread::sleep(std::time::Duration::from_secs(10));
            let mut db = db_binding.lock().unwrap();
            let now = SystemTime::now();
            db.retain(|_, item| now.duration_since(item.created_at).unwrap().as_secs() < 900);
        });
        AxumState {
            db: Arc::clone(&db),
        }
    }

    pub fn get(&self, key: String) -> Option<String> {
        let db = self.db.lock().unwrap();
        if let Some(item) = db.get(&key) {
            Some(item.verifier.clone())
        } else {
            None
        }
    }

    pub fn set(&self, key: String, value: String) {
        let mut db = self.db.lock().unwrap();
        db.insert(
            key,
            ItemOauthAxum {
                verifier: value,
                created_at: SystemTime::now(),
            },
        );
    }

    pub fn get_all_items(&self) -> Vec<ItemOauthAxum> {
        let db = self.db.lock().unwrap();
        db.values().cloned().collect::<Vec<ItemOauthAxum>>()
    }
}
