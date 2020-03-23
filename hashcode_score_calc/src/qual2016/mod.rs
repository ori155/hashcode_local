mod parsing;
mod error;

use error::Qual2016ScoringError;
use crate::{ScoringError, InputFileName, Score};


type Row = u16;
type Col = u16;
type Turn = u32;
type WarehouseID = u16;
type DroneID = u16;
type Weight = u16;
type ProductID = u16;
type OrderID = u16;
type CommandNumber = u64; // DroneID X Turn
type WarehouseProductInventory = u16;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct MapSize {
    pub rows: Row,
    pub cols: Col
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Location {
    pub row: Row,
    pub col: Col
}

impl Location {
    pub fn flight_time(&self, other: &Location) -> Turn {
        if self == other {
            return 0
        }
        let diff_row = if self.row > other.row {self.row - other.row} else {other.row - self.row};
        let diff_col = if self.col > other.col {self.col - other.col} else {other.col - self.col};
        ((diff_row as f32).powi(2) + (diff_col as f32).powi(2)).sqrt().ceil() as Turn
    }
}

#[cfg(test)]
mod tests {
    use super::Location;
    #[test]
    fn test_flight_time_rows() {
        let a = Location{row: 0, col:0};
        let b = Location{row: 1, col:0};
        assert_eq!(a.flight_time(&b), 1);
    }

    #[test]
    fn test_flight_time_diag() {
        let a = Location{row: 0, col:0};
        let b = Location{row: 5, col:5};
        assert_eq!(a.flight_time(&b), 8);
    }
}


#[derive(Clone, Debug, Hash)]
pub struct Product {
    id: ProductID,
    weight: Weight
}

impl PartialEq for Product {
    fn eq(&self, other: &Product) -> bool {
        self.id == other.id
    }
}
impl Eq for Product {}

#[derive(Clone, Debug)]
pub struct Order {
    pub id: OrderID,
    pub location: Location,
    pub products: Vec<ProductID>
}

impl Order {
    pub fn supply(&mut self, product_id: ProductID, number_of_items: WarehouseProductInventory) -> Result<(), Qual2016ScoringError> {

        // TODO: this is a shitty way... maybe use hasmap
        for _ in 0..number_of_items {
            let index_to_remove = self.products.iter()
                .find_position(|pid| **pid == product_id)
                .ok_or(Qual2016ScoringError::OverSupplyingOrder {order_id: self.id})?.0;
            self.products.remove(index_to_remove);
        }

        Ok(())

    }
    pub fn is_done(&self) -> bool {
        self.products.len() == 0
    }
}

impl Command {
    pub fn get_drone_id(&self) -> DroneID{
        use Command::*;
        match self {
            Load{ drone_id, .. } => *drone_id,
            Unload{ drone_id, .. } => *drone_id,
            Deliver{ drone_id, .. } => *drone_id,
            Wait{ drone_id, .. } => *drone_id,
            GeneratedFlight {drone_id, .. } => *drone_id
        }
    }

    pub fn get_location(&self, earth: &Earth) -> Result<Option<Location>, Qual2016ScoringError> {
        use Command::*;
        Ok(
            match self {
                Load{ warehouse_id, .. } => Some(earth.get_warehouse(*warehouse_id)?.location),
                Unload{ warehouse_id, .. } => Some(earth.get_warehouse(*warehouse_id)?.location),
                Deliver{ order_id, .. } => Some(earth.get_order(*order_id)?.location),
                GeneratedFlight {to, ..} => Some(*to),
                _ => None
            }
        )

    }
}


#[derive(Debug, Clone)]
pub struct Warehouse {
    pub id: WarehouseID,
    pub location: Location,
    pub inventory: Vec<ProductID>,
}

impl Warehouse {
    pub fn take_out_product(&mut self, product_id: ProductID, number_of_products: WarehouseProductInventory) -> Result<(), Qual2016ScoringError> {
        let inv = self.inventory.get_mut(product_id as usize)
            .ok_or(Qual2016ScoringError::UnknownProduct {product_id})?;
        if *inv >= number_of_products {
            *inv -= number_of_products;
            Ok(())
        } else {
            Err(Qual2016ScoringError::OverTakingWarehouse {warehouse_id: self.id})
        }
    }

    pub fn insert_product(&mut self, prod_id: ProductID, number_of_products: WarehouseProductInventory) {
        self.inventory[prod_id as usize] += number_of_products;
    }
}

pub struct Case {
    _map: MapSize,
    warehouses: Vec<Warehouse>,
    total_turns: Turn,
    number_of_drones: DroneID,
    max_payload: Weight,
    orders: Vec<Order>,
    products: Vec<Product>
}

impl Case {
    pub fn parse(input: &'static str) -> Result<Self, ScoringError> {
        parsing::parse_input_file(input)
            .map(|(_input, case)| case)
            .map_err(|e| ScoringError::InputFileError(Box::new(e)))
    }

    pub fn get_earth_bound(&self) -> Earth {
        Earth{ warehouses: self.warehouses.clone(), orders: self.orders.clone() }
    }

    pub fn get_product(&self, product_id: ProductID) -> Result<&Product, Qual2016ScoringError> {
        self.products.get(product_id as usize)
            .ok_or(Qual2016ScoringError::UnknownProduct {product_id})
    }

}

pub struct Earth {
    warehouses: Vec<Warehouse>,
    orders: Vec<Order>
}

impl Earth {
    pub fn get_warehouse(&self, warehouse_id: WarehouseID) -> Result<&Warehouse, Qual2016ScoringError> {
        self.warehouses.get(warehouse_id as usize)
            .ok_or(Qual2016ScoringError::UnknownWarehouse {warehouse_id})
    }

    pub fn get_mut_warehouse(&mut self, warehouse_id: WarehouseID) -> Result<&mut Warehouse, Qual2016ScoringError> {
        self.warehouses.get_mut(warehouse_id as usize)
            .ok_or(Qual2016ScoringError::UnknownWarehouse {warehouse_id})
    }

    pub fn get_order(&self, order_id: OrderID) -> Result<&Order, Qual2016ScoringError> {
        self.orders.get(order_id as usize)
            .ok_or(Qual2016ScoringError::UnknownOrder {order_id})
    }

    pub fn get_mut_order(&mut self, order_id: OrderID) -> Result<&mut Order, Qual2016ScoringError> {
        self.orders.get_mut(order_id as usize)
            .ok_or(Qual2016ScoringError::UnknownOrder {order_id})
    }
}

lazy_static!{
    static ref CASE_EXAMPLE: Case = Case::parse(include_str!("../../assets/2016qual/inputs/example.in")).
                                        unwrap();
    static ref CASE_BUSY_DAY: Case = Case::parse(include_str!("../../assets/2016qual/inputs/busy_day.in")).
                                        unwrap();
    static ref CASE_MOTHER_OF_ALL_WAREHOUSES: Case = Case::parse(include_str!("../../assets/2016qual/inputs/mother_of_all_warehouses.in")).
                                        unwrap();
    static ref CASE_REDUNDANCY: Case = Case::parse(include_str!("../../assets/2016qual/inputs/redundancy.in")).
                                        unwrap();
}

#[derive(Debug)]
struct ExecutedCommand<'drone, 'case> {
    command: Command,
    on_drone: &'drone mut Drone<'case>,
}

impl ExecutedCommand<'_, '_> {
    pub fn is_unload(&self) -> bool {
        match &self.command {
            &Command::Unload {..} => true,
            _ => false
        }
    }
}

use std::collections::{VecDeque, HashMap};
use itertools::Itertools;
use serde::export::fmt::Debug;

struct DroneEarthInteraction<'case, 'drone, 'earth> {
    drone: &'drone mut Drone<'case>,
    earth: &'earth mut Earth
}

impl DroneEarthInteraction<'_, '_, '_> {
    /// retruns time it took for the command, flying+command
    /// updates the location of the drone, and the time for the next command
    pub fn load(&mut self, warehouse_id: WarehouseID, product_id: ProductID, amount: WarehouseProductInventory) -> Result<(), Qual2016ScoringError> {
        let warehouse = self.earth.get_mut_warehouse(warehouse_id)?;
        let product = self.drone.case.get_product(product_id)?;

        // Self check
        if warehouse.location != self.drone.location {
            return Err(Qual2016ScoringError::DroneNotInWarehouse {drone_id: self.drone.id, warehouse_id});
        }

        if self.drone.room_left()? < (product.weight * amount) {
            return Err(Qual2016ScoringError::DronePassedWeightLimit { drone_id: self.drone.id });
        }

        warehouse.take_out_product(product_id, amount)?;
        let drone_product_slot = self.drone.carrying.entry(product_id).or_insert(0);
        *drone_product_slot += amount;

        self.drone.time_for_next_command += 1;

        Ok(())
    }

    pub fn unload(&mut self, warehouse_id: WarehouseID, product_id: ProductID, amount: WarehouseProductInventory) -> Result<(), Qual2016ScoringError> {
        let warehouse = self.earth.get_mut_warehouse(warehouse_id)?;
        let drone_product_slot = self.drone.carrying.get_mut(&product_id)
            .ok_or(Qual2016ScoringError::UnknownProduct {product_id})?;

        // Self check
        if warehouse.location != self.drone.location {
            return Err(Qual2016ScoringError::DroneNotInWarehouse {drone_id: self.drone.id, warehouse_id});
        }

        if *drone_product_slot < amount {
            Err(Qual2016ScoringError::OverTakingDrone { drone_id: self.drone.id, product_id })
        } else {
            warehouse.insert_product(product_id, amount);

            self.drone.time_for_next_command += 1;

            Ok(())
        }

    }

    pub fn deliver(&mut self, order_id: OrderID, product_id: ProductID, amount: WarehouseProductInventory) -> Result<(), Qual2016ScoringError> {
        let order = self.earth.get_mut_order(order_id)?;

        let drone_product_slot = self.drone.carrying.get_mut(&product_id)
            .ok_or(Qual2016ScoringError::UnknownProduct {product_id})?;

        // Self check
        if order.location != self.drone.location {
            return Err(Qual2016ScoringError::DroneNotInOrderLocation {drone_id: self.drone.id, order_id});
        }

        if *drone_product_slot < amount {
            Err(Qual2016ScoringError::OverTakingDrone { drone_id: self.drone.id, product_id })
        } else {
            order.supply(product_id, amount)?;
            *drone_product_slot -= amount;

            self.drone.time_for_next_command += 1;

            Ok(())
        }
    }
}

struct Drone<'case> {
    id: DroneID,
    to_execute: VecDeque<Command>,
    location: Location,
    carrying: HashMap<ProductID, WarehouseProductInventory>,
    case: &'case Case,
    // When time_for_next_command arrives (t == time_for...), then the drone needs to execute the command in to_execute
    // and update the time for the next command
    time_for_next_command: Turn
}

use std::fmt;
impl Debug for Drone<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "Drone #{:?}, next_time: {:?}, commands: {:?}, location: {:?}, carrying: {:?}",
                    self.id, self.time_for_next_command, self.to_execute, self.location, self.carrying)
    }
}

impl<'case> Drone<'case> {
    pub fn new(id: DroneID, location: Location, case: &'case Case) -> Self {
        Self{
            id,
            to_execute: VecDeque::new(),
            location,
            case,
            carrying: HashMap::new(),
            time_for_next_command: 0
        }
    }

    pub fn register_command(&mut self, command: Command, earth: &Earth) -> Result<(), Qual2016ScoringError> {
        let flight_command: Option<Command> = {
            // last registered location or self.location if start
            // iter backwards in case some of the commands are wait (no location)
            let mut last_location = None;
            for last_command_with_location in self.to_execute.iter().rev() {
                if let Some(recent_last_location) = last_command_with_location.get_location(&earth)? {
                    last_location = Some(recent_last_location);
                    break;
                }
            }

            let last_location = last_location.unwrap_or(self.location);

            if let Some(next_location) = command.get_location(&earth)? {
                if next_location != last_location {
                    Some(Command::GeneratedFlight {to: next_location, drone_id: self.id})
                } else { None }
            } else { None }
        };

        if let Some(f_cmd) = flight_command {
            self.to_execute.push_back(f_cmd);
        }

        self.to_execute.push_back(command);
        Ok(())
    }

    pub fn fly_to(&mut self, location: Location) {
        self.time_for_next_command += self.location.flight_time(&location);
        self.location = location;
    }

    pub fn carried_weight(&self) -> Result<Weight, Qual2016ScoringError> {
        self.carrying.iter()
            .map(|(product_id,amount): (&ProductID, &u16) | {
                self.case.products
                    .get(*product_id as usize)
                    .ok_or(Qual2016ScoringError::UnknownProduct {product_id: *product_id})
                    .map(|product: &Product| product.weight * (*amount))
            })
            .sum::<Result<Weight, _>>()
    }

    pub fn room_left(&self) -> Result<Weight, Qual2016ScoringError> {
        // Should always have room left (or zero), else this method will underflow
        let weight_limit = self.case.max_payload;
        let carried_weight = self.carried_weight()?;
        Ok(weight_limit - carried_weight)
    }

    pub fn on_earth<'drone, 'earth>(&'drone mut self, earth: &'earth mut Earth) -> DroneEarthInteraction<'case, 'drone, 'earth> {
        DroneEarthInteraction { drone: self, earth }
    }

    pub fn is_time_for_next_command(&self, time: Turn) -> bool {
        self.time_for_next_command == time
    }

    pub fn get_command_to_execute<'drone>(&'drone mut self, at_time: Turn) -> Option<ExecutedCommand<'drone, 'case>> {
        if self.is_time_for_next_command(at_time) {
            if let Some(command) = self.to_execute.pop_front() {
                Some(ExecutedCommand::<'drone, 'case>{
                    command ,
                    on_drone: self})
            } else {None}
        } else { None }
    }

    pub fn wait(&mut self, turns_to_wait: Turn) {
        self.time_for_next_command += turns_to_wait;
    }
}

#[derive(Debug)]
pub enum Command {
    Load {
        drone_id: DroneID,
        warehouse_id: WarehouseID,
        product_id: ProductID,
        number_of_items: WarehouseProductInventory
    },
    Unload {
        drone_id: DroneID,
        warehouse_id: WarehouseID,
        product_id: ProductID,
        number_of_items: WarehouseProductInventory
    },
    Deliver {
        drone_id: DroneID,
        order_id: OrderID,
        product_id: ProductID,
        number_of_items: WarehouseProductInventory
    },
    Wait {
        drone_id: DroneID,
        turns: Turn
    },
    GeneratedFlight {
        drone_id: DroneID,
        to: Location
    }
}

pub fn score(submission: &str, case: &InputFileName) -> Result<Score, ScoringError> {
    let case: &Case = match case {
        InputFileName(ref s) if s.starts_with("example") => &*CASE_EXAMPLE,
        InputFileName(ref s) if s.starts_with("busy_day") => &*CASE_BUSY_DAY,
        InputFileName(ref s) if s.starts_with("mother_of_all_warehouses") => &*CASE_MOTHER_OF_ALL_WAREHOUSES,
        InputFileName(ref s) if s.starts_with("redundancy") => &*CASE_REDUNDANCY,
        input_case @ InputFileName(_) => return Err(ScoringError::UnknownInputCase(input_case.clone()))
    };

    let mut drones: Vec<Drone> = (0..case.number_of_drones).into_iter()
        .map(|i| Drone::new(i, case.warehouses[0].location, case))
        .collect();

    let commands: Vec<Command> = parsing::parse_submission(submission)
        .map(|(_submission, commands)| commands)
        .map_err(|e| ScoringError::SubmissionFileError(Box::new(e.to_owned())))?;

    // The right place to check that would be the constructor or the parser, but...
    for command in &commands {
        let drone_id = command.get_drone_id();
        match command {
            Command::Load { number_of_items, .. } => { if *number_of_items == 0 {return Err(Qual2016ScoringError::CommandWithAmountZero{drone_id}.into())}},
            Command::Unload { number_of_items,.. } => { if *number_of_items == 0 {return Err(Qual2016ScoringError::CommandWithAmountZero{drone_id}.into())}},
            Command::Deliver { number_of_items,.. } => { if *number_of_items == 0 {return Err(Qual2016ScoringError::CommandWithAmountZero{drone_id}.into())}},
            _ => {}
        }
    }

    let mut earth = case.get_earth_bound();

    for command in commands {
        let drone_id = command.get_drone_id();
        let drone: &mut Drone = drones.get_mut(drone_id as usize)
            .ok_or(Qual2016ScoringError::CommandIssuedToUnknownDrone {drone_id})?;
        drone.register_command(command, &earth);
    }


    let mut submission_score: Score = 0;
    for t in 0..case.total_turns{
        let commands_to_execute_unload_first = {
            let commands_to_execute = drones.iter_mut()
                .filter_map(|d| d.get_command_to_execute(t));

            let (mut unload_commands, other_commands): (Vec<_>, Vec<_>) = commands_to_execute
                .partition(|exec_cmd| exec_cmd.is_unload());
            unload_commands.extend(other_commands.into_iter());
            unload_commands
        };

        for exec_command in commands_to_execute_unload_first{
            match exec_command.command {
                Command::GeneratedFlight {to, ..} => {
                    exec_command.on_drone
                        .fly_to(to);
                },
                Command::Load { warehouse_id, product_id, number_of_items, .. } => {
                    exec_command.on_drone
                        .on_earth(&mut earth)
                        .load(warehouse_id, product_id, number_of_items)?;
                },
                Command::Unload { warehouse_id, product_id, number_of_items, .. } => {
                    exec_command.on_drone
                        .on_earth(&mut earth)
                        .unload(warehouse_id, product_id, number_of_items)?;
                },
                Command::Deliver { order_id, product_id, number_of_items, .. } => {
                    exec_command.on_drone
                        .on_earth(&mut earth)
                        .deliver(order_id, product_id, number_of_items)?;

                    if earth.get_order(order_id)?.is_done() {
                        let added_score = ((case.total_turns as Score - t as Score) * 100) / case.total_turns as Score;
                        let should_round_up = (((case.total_turns as Score - t as Score) * 100) % case.total_turns as Score) != 0;

                        submission_score += added_score + if should_round_up {1} else {0};
                    }
                },
                Command::Wait { turns, ..} => {
                    exec_command.on_drone.wait(turns);
                },
            }
        }

    }
    Ok(submission_score)
}
