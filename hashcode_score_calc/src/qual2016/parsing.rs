use nom::{IResult, combinator::map_res, character::complete::digit1};
use nom::sequence::tuple;
use nom::multi::many_m_n;
use nom::character::complete::multispace0;
use nom::sequence::terminated;

use super::{Row, Col, DroneID, Turn, Weight, ProductID, Product,
            Warehouse, WarehouseID, Location, WarehouseProductInventory,
            Order, OrderID, MapSize};
use std::str::FromStr;

fn decimal_number<N: FromStr>(input: &str) -> IResult<&str, N> {
    map_res(digit1, |s: &str| s.parse::<N>())(input)
}

fn decimal_number_ms<N: FromStr>(input: &str) -> IResult<&str, N> {
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

fn one_order_loc_and_prods(input: &str) -> IResult<&str, (Location, Vec<ProductID>)> {
    let (input, ord_location) = location(input)?;
    let (input, number_of_items) = decimal_number_ms::<ProductID>(input)?;
    let (input, products) = many_m_n(number_of_items as usize,
                                   number_of_items as usize,
                                   decimal_number_ms::<ProductID>)(input)?;

    Ok((input, (ord_location, products)))
}

fn orders(input: &str) -> IResult<&str, Vec<Order>> {
    let (input, number_of_orders) = decimal_number_ms::<OrderID>(input)?;
    let (input, locs_and_prods) = many_m_n(number_of_orders as usize,
                                   number_of_orders as usize,
                                    one_order_loc_and_prods)(input)?;

    let orders = locs_and_prods
        .into_iter()
        .enumerate()
        .map(|(i, (loc, prods))| Order{id:i as ProductID, location: loc, products: prods})
        .collect();
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
        _map: MapSize {rows, cols},
        warehouses: case_warehouses,
        products: case_products,
        total_turns: turns,
        number_of_drones: drones,
        max_payload,
        orders: case_orders
    }))
}

use nom::character::complete::one_of;

fn one_command(input: &str) -> IResult<&str, Command> {
    let (input, drone_id) = decimal_number_ms::<DroneID>(input)?;
    let (input, command_type) = terminated(one_of("LUDW"), multispace0)(input)?;
    match command_type {
        'L' => {
           let (input, (warehouse_id, product_id, number_of_items)) = tuple((
               decimal_number_ms::<WarehouseID>,
               decimal_number_ms::<ProductID>,
               decimal_number_ms::<WarehouseProductInventory>,
           ))(input)?;
           Ok((input, Command::Load {
               drone_id,
               warehouse_id,
               product_id,
               number_of_items
           }))
        },
        'U' => {
            let (input, (warehouse_id, product_id, number_of_items)) = tuple((
                decimal_number_ms::<WarehouseID>,
                decimal_number_ms::<ProductID>,
                decimal_number_ms::<WarehouseID>,
            ))(input)?;
            Ok((input, Command::Unload {
                drone_id,
                warehouse_id,
                product_id,
                number_of_items
            }))

        },
        'D' => {
            let (input, (order_id, product_id, number_of_items)) = tuple((
                decimal_number_ms::<OrderID>,
                decimal_number_ms::<ProductID>,
                decimal_number_ms::<WarehouseID>,
            ))(input)?;

            Ok((input, Command::Deliver {
                drone_id,
                order_id,
                product_id,
                number_of_items
            }))

        },
        'W' => {
            let (input, turns) = decimal_number_ms::<Turn>(input)?;

            Ok((input, Command::Wait { drone_id, turns }))

        },
        _ => unreachable!("Already checked that this is a known command")
    }
}

use super::Command;
use super::CommandNumber;

pub fn parse_submission(input: &str) -> IResult<&str, Vec<Command>> {
    let (input, number_of_commands) = decimal_number_ms::<CommandNumber>(input)?;
    many_m_n(number_of_commands as usize,
             number_of_commands as usize,
             one_command)(input)
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
