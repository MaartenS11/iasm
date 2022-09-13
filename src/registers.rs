use std::{collections::HashMap, ops::{Index, IndexMut}};

pub struct Registers<'a> {
    pub variables: HashMap<&'a str, i64>
}

impl<'a> Registers<'a> {
    pub fn new() -> Self {
        let mut variables: HashMap<&str, i64> = HashMap::new();
        variables.insert("zero", 0);
        variables.insert("ra", Self::random_data());
        variables.insert("sp", Self::random_data());
        variables.insert("eip", Self::random_data());

        variables.insert("t0", Self::random_data());
        variables.insert("t1", Self::random_data());
        variables.insert("t2", Self::random_data());

        variables.insert("s0", Self::random_data());
        variables.insert("s1", Self::random_data());

        variables.insert("a0", Self::random_data());
        variables.insert("a1", Self::random_data());
        variables.insert("a2", Self::random_data());
        variables.insert("a3", Self::random_data());
        variables.insert("a4", Self::random_data());
        variables.insert("a5", Self::random_data());
        variables.insert("a6", Self::random_data());
        variables.insert("a7", Self::random_data());

        variables.insert("s2", Self::random_data());
        variables.insert("s3", Self::random_data());
        variables.insert("s4", Self::random_data());
        variables.insert("s5", Self::random_data());
        variables.insert("s6", Self::random_data());
        variables.insert("s7", Self::random_data());
        variables.insert("s8", Self::random_data());
        variables.insert("s9", Self::random_data());
        variables.insert("s10", Self::random_data());
        variables.insert("s11", Self::random_data());

        variables.insert("t3", Self::random_data());
        variables.insert("t4", Self::random_data());
        variables.insert("t5", Self::random_data());
        variables.insert("t6", Self::random_data());

        Registers {
            variables: variables
        }
    }

    fn random_data() -> i64 {
        let ptr = Box::into_raw(Box::new(123));
        return ptr as i64;
    }

    pub fn has_register(self: &Registers<'a>, register: &str) -> bool {
        self.variables.contains_key(register)
    }
}

impl Index<&str> for Registers<'_> {
    type Output = i64;

    fn index(&self, register: &str) -> &Self::Output {
        &self.variables[register]
    }
}

impl IndexMut<&str> for Registers<'_> {
    fn index_mut(&mut self, register: &str) -> &mut Self::Output {
        &mut *self.variables.get_mut(register).unwrap()
    }
}
