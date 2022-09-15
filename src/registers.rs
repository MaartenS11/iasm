use std::{collections::HashMap, ops::{Index, IndexMut}};

pub struct Registers {
    pub variables: HashMap<String, i64>
}

impl Registers {
    pub fn new() -> Self {
        let mut variables: HashMap<String, i64> = HashMap::new();
        variables.insert("zero".to_owned(), 0);
        variables.insert("ra".to_owned(), Self::random_data());
        variables.insert("sp".to_owned(), Self::random_data());
        variables.insert("eip".to_owned(), Self::random_data());

        variables.insert("t0".to_owned(), Self::random_data());
        variables.insert("t1".to_owned(), Self::random_data());
        variables.insert("t2".to_owned(), Self::random_data());

        variables.insert("s0".to_owned(), Self::random_data());
        variables.insert("s1".to_owned(), Self::random_data());

        variables.insert("a0".to_owned(), Self::random_data());
        variables.insert("a1".to_owned(), Self::random_data());
        variables.insert("a2".to_owned(), Self::random_data());
        variables.insert("a3".to_owned(), Self::random_data());
        variables.insert("a4".to_owned(), Self::random_data());
        variables.insert("a5".to_owned(), Self::random_data());
        variables.insert("a6".to_owned(), Self::random_data());
        variables.insert("a7".to_owned(), Self::random_data());

        variables.insert("s2".to_owned(), Self::random_data());
        variables.insert("s3".to_owned(), Self::random_data());
        variables.insert("s4".to_owned(), Self::random_data());
        variables.insert("s5".to_owned(), Self::random_data());
        variables.insert("s6".to_owned(), Self::random_data());
        variables.insert("s7".to_owned(), Self::random_data());
        variables.insert("s8".to_owned(), Self::random_data());
        variables.insert("s9".to_owned(), Self::random_data());
        variables.insert("s10".to_owned(), Self::random_data());
        variables.insert("s11".to_owned(), Self::random_data());

        variables.insert("t3".to_owned(), Self::random_data());
        variables.insert("t4".to_owned(), Self::random_data());
        variables.insert("t5".to_owned(), Self::random_data());
        variables.insert("t6".to_owned(), Self::random_data());

        Registers {
            variables: variables
        }
    }

    fn random_data() -> i64 {
        let ptr = Box::into_raw(Box::new(123));
        return ptr as i64;
    }

    pub fn has_register(&self, register: &str) -> bool {
        self.variables.contains_key(register)
    }
}

impl Index<&str> for Registers {
    type Output = i64;

    fn index(&self, register: &str) -> &Self::Output {
        &self.variables[register]
    }
}

impl IndexMut<&str> for Registers {
    fn index_mut(&mut self, register: &str) -> &mut Self::Output {
        &mut *self.variables.get_mut(register).unwrap()
    }
}
