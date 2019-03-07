use prettytable::{Row, Cell};


#[derive(Debug, Deserialize, Clone)]
pub struct User {
    pub email: String,
    pub user_id: String,
    pub nickname: Option<String>, // unsure if should be optional.
    pub last_login: Option<String>,
}


impl User {
    pub fn to_table_row(&self) -> Row {
        let nickname = self.nickname.clone().unwrap_or("".to_string());
        let last_login = self.last_login.clone().unwrap_or("".to_string());
        Row::new(vec![
            Cell::new(&self.email),
            Cell::new(&self.user_id),
            Cell::new(&nickname),
            Cell::new(&last_login)
        ])
    }

    pub fn matches(&self, pattern: &str) -> bool {
        match (self.email.find(pattern), self.user_id.find(pattern)) {
            (None, None) => false,
            _ => true
        }
    }
}


#[cfg(test)]
mod test {
    use crate::user::User;

    fn new_user(email: &str, id: &str) -> User {
        User {
            email: email.to_string(),
            user_id: id.to_string(),
            nickname: None,
            last_login: None
        }
    }

    #[test]
    fn double_two() {
        assert_eq!(2 * 2, 4);
    }

    #[test]
    fn matches_exact() {
        let user = new_user("user@email.test", "2");
        assert_eq!(true, user.matches("user@email.test"));
        assert_eq!(true, user.matches("2"));
    }

    #[test]
    fn matches_partial() {
        let user = new_user("user@email.test", "a1b2c3");
        assert_eq!(true, user.matches(".test"));
        assert_eq!(true, user.matches("email.test"));
        assert_eq!(true, user.matches("user"));
        assert_eq!(true, user.matches("a1b"));
        assert_eq!(true, user.matches("2c3"));
    }
}