#[derive(Debug)]
struct Values {
}
impl Values {
    fn new() -> Self { Self {
    } }
fn operator_eq(&self, other: &Values) -> bool {
    return true;
}
fn operator_lt(&self, other: &Values) -> bool {
    return false;
}
fn operator_index(&self, index: i64) -> &i64 {
     todo!() 
}
}
impl PartialEq for Values {
    fn eq(&self, rhs: &Self) -> bool { self.operator_eq(rhs) }
}
impl std::cmp::PartialOrd for Values {
    fn partial_cmp(&self, rhs: &Self) -> Option<std::cmp::Ordering> { if self.operator_lt(rhs) { Some(std::cmp::Ordering::Less) } else if self.operator_eq(rhs) { Some(std::cmp::Ordering::Equal) } else None }
}
impl std::ops::Index<i64> for Values {
    type Output = i64;
    fn index(&self, index: i64) -> &Self::Output { self.operator_index(index) }
}
