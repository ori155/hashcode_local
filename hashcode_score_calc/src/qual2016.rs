use thiserror::Error;
use crate::{ScoringError, InputFileName, Score};


#[derive(Error, Debug, PartialEq, Eq)]
pub enum Qual2016ScoringError {
    #[error("Tried to access location row: {row}, col: {col} which is out of bounds")]
    LocationOutOfMap {row: Row, col: Col}
}


impl From<Qual2016ScoringError> for ScoringError {
    fn from(e: Qual2016ScoringError) -> Self {
        ScoringError::ChallengeSpecific(Box::new(e))
    }
}

type Row = u16;
type Col = u16;
type Turn = u32;
type WarehouseID = u16;
type DroneID = u16;
type Weight = u16;
type ProductID = u16;
type OrderID = u16;
type WarehouseProductInventory = u16;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
struct MapSize {
    pub rows: Row,
    pub cols: Col
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
struct Location {
    pub row: Row,
    pub col: Col
}

impl MapSize {
    pub fn loc(&self, row: Row, col: Col) -> Result<Location, Qual2016ScoringError> {
        if row < self.rows && col < self.cols {
            Ok(Location{ row, col })
        } else {
            Err(Qual2016ScoringError::LocationOutOfMap { row, col })
        }
    }
}

struct Product {
    id: ProductID,
    weight: Weight
}

struct Order {
    pub location: Location,
    pub products: Vec<ProductID>
}

mod parsing {
    use nom::{IResult, combinator::map_res, character::complete::digit1};
    use nom::sequence::tuple;
    use nom::multi::many_m_n;
    use super::{Row, Col, DroneID, Turn, Weight, ProductID, Product,
                Warehouse, WarehouseID, Location, WarehouseProductInventory,
                Order, OrderID, MapSize};
    use std::str::FromStr;

    fn decimal_number<N: FromStr>(input: &str) -> IResult<&str, N> {
        map_res(digit1, |s: &str| s.parse::<N>())(input)
    }

    fn decimal_number_ms<N: FromStr>(input: &str) -> IResult<&str, N> {
        use nom::character::complete::multispace0;
        use nom::sequence::terminated;
        terminated(decimal_number, multispace0)(input)
    }

    fn first_line(input: &str) -> IResult<&str, (Row, Col, DroneID, Turn, Weight)> {
        tuple((decimal_number_ms, decimal_number_ms, decimal_number_ms, decimal_number_ms, decimal_number_ms))(input)
    }

    fn products(input: &str) -> IResult<&str, Vec<Product>> {
        use std::convert::TryFrom;
        let (input, number_of_products) = decimal_number_ms::<ProductID>(input)?;
        let (input, weights) = many_m_n(number_of_products as usize,
                                        number_of_products as usize,
                                        decimal_number_ms::<Weight>)
            (input)?;
        Ok((input, weights.into_iter()
            .enumerate()
            .map(|(i, w)| Product{ id: ProductID::try_from(i).expect("ProdId too big"), weight: w })
            .collect()
        ))
    }

    fn location(input: &str) -> IResult<&str, Location> {
        let (input, (r, c)): (&str, (Row, Col)) = tuple((decimal_number_ms, decimal_number_ms))(input)?;
        Ok((input, Location{ row: r, col: c }))
    }

    fn inventory_of_size(number_of_products: ProductID) -> impl Fn(&str) -> IResult<&str, Vec<WarehouseProductInventory>> {
        move |input :&str| -> IResult<&str, Vec<WarehouseProductInventory>> {
            many_m_n(number_of_products as usize,
                     number_of_products as usize,
                     decimal_number_ms::<WarehouseProductInventory>)(input)
        }

    }

    fn warehouses_with_inventory_size(inventory_size: ProductID) -> impl Fn(&str) -> IResult<&str, Vec<Warehouse>> {
        use std::convert::TryFrom;
        move |input :&str| -> IResult<&str, Vec<Warehouse>> {
            let (input, number_of_warehouses) = decimal_number_ms::<ProductID>(input)?;
            let (input, locations) = many_m_n(number_of_warehouses as usize,
                                            number_of_warehouses as usize,
                                            tuple((
                                                location,
                                                inventory_of_size(inventory_size)
                                            ))
            )(input)?;

            Ok((input, locations.into_iter()
                .enumerate()
                .map(|(i, (location, inventory))| Warehouse{ id: WarehouseID::try_from(i)
                    .expect("ProdId too big"),
                    location,
                    inventory
                })
                .collect()
            ))
        }
    }

    fn one_order(input: &str) -> IResult<&str, Order> {
        let (input, ord_location) = location(input)?;
        let (input, number_of_items) = decimal_number_ms::<ProductID>(input)?;
        let (input, products) = many_m_n(number_of_items as usize,
                                       number_of_items as usize,
                                       decimal_number_ms::<ProductID>)(input)?;

        Ok((input, Order{ location: ord_location, products }))
    }

    fn orders(input: &str) -> IResult<&str, Vec<Order>> {
        let (input, number_of_orders) = decimal_number_ms::<OrderID>(input)?;
        let (input, orders) = many_m_n(number_of_orders as usize,
                                       number_of_orders as usize,
                                        one_order)(input)?;

        Ok((input, orders))

    }

    use super::Case;
    use std::convert::TryInto;
    pub(crate) fn parse_input_file(input: &str) -> IResult<&str, Case> {
        let (input, (rows, cols, drones, turns, max_payload)) = first_line(input)?;
        let (input, case_products) = products(input)?;
        let (input, case_warehouses) = warehouses_with_inventory_size(case_products.len().try_into().unwrap())
            (input)?;
        let (input, case_orders) = orders(input)?;

        Ok((input, Case{
            map: MapSize {rows, cols},
            warehouses: case_warehouses,
            total_turns: turns,
            number_of_drones: drones,
            max_payload
        }))
    }

    #[cfg(test)]
    mod tests {
        use super::{decimal_number, first_line, products, warehouses_with_inventory_size,
                    orders, Location};
        use crate::qual2016::Row;

        #[test]
        fn decimal_number_works() {
            let res = decimal_number::<Row>("123");
            assert_eq!(res, Ok(("", 123)));
        }

        #[test]
        fn test_first_line() {
            let res = first_line("100 100 3 50 500");
            assert_eq!(res, Ok(("", (100, 100, 3, 50, 500))));
        }

        #[test]
        fn test_products() {
            let (input, prod) = products("3\n1 2 3\n").expect("should be ok");
            assert_eq!(input, "");
            assert_eq!(prod.len(), 3);
            assert_eq!(prod[0].weight, 1);
            assert_eq!(prod[1].weight, 2);
            assert_eq!(prod[2].weight, 3);
        }

        #[test]
        fn test_warehouses() {
            let (input, warehouses) = warehouses_with_inventory_size(3)("1\n1 2\n4 5 6\n").expect("should be ok");
            assert_eq!(input, "");
            assert_eq!(warehouses.len(), 1);
            assert_eq!(warehouses[0].inventory, vec![4,5,6]);
            assert_eq!(warehouses[0].location, Location{row: 1, col: 2});
        }

        #[test]
        fn test_orders() {
            let (input, ord) = orders("2\n1 2\n2\n3 4\n3 3\n1\n5\n").expect("should be ok");
            assert_eq!(input, "");
            assert_eq!(ord.len(), 2);
            assert_eq!(ord[0].products, vec![3, 4]);
            assert_eq!(ord[0].location, Location{row: 1, col: 2});
            assert_eq!(ord[1].products, vec![5]);
            assert_eq!(ord[1].location, Location{row: 3, col: 3});
        }
    }
}

#[derive(Clone, Debug)]
struct Warehouse {
    pub id: WarehouseID,
    pub location: Location,
    pub inventory: Vec<WarehouseProductInventory>,
}

pub struct Case {
    map: MapSize,
    warehouses: Vec<Warehouse>,
    total_turns: Turn,
    number_of_drones: DroneID,
    max_payload: Weight
}

impl Case {
    fn parse(input: &'static str) -> Result<Self, ScoringError> {
        parsing::parse_input_file(input)
            .map(|(input, case)| case)
            .map_err(|e| ScoringError::InputFileError(Box::new(e)))
    }
}

lazy_static!{
    static ref CASE_EXAMPLE: Case = Case::parse(include_str!("../assets/2016qual/inputs/example.in")).
                                        unwrap();
    static ref CASE_BUSY_DAY: Case = Case::parse(include_str!("../assets/2016qual/inputs/busy_day.in")).
                                        unwrap();
    static ref CASE_MOTHER_OF_ALL_WAREHOUSES: Case = Case::parse(include_str!("../assets/2016qual/inputs/mother_of_all_warehouses.in")).
                                        unwrap();
    static ref CASE_REDUNDANCY: Case = Case::parse(include_str!("../assets/2016qual/inputs/redundancy.in")).
                                        unwrap();
}

pub fn score(submission: &str, case: &InputFileName) -> Result<Score, ScoringError> {
    let case: &Case = match case {
        InputFileName(ref s) if s.starts_with("example") => &*CASE_EXAMPLE,
        InputFileName(ref s) if s.starts_with("busy_day") => &*CASE_BUSY_DAY,
        InputFileName(ref s) if s.starts_with("mother_of_all_warehouses") => &*CASE_MOTHER_OF_ALL_WAREHOUSES,
        InputFileName(ref s) if s.starts_with("redundancy") => &*CASE_REDUNDANCY,
        input_case @ InputFileName(_) => return Err(ScoringError::UnknownInputCase(input_case.clone()))
    };
   Ok(0)
}
