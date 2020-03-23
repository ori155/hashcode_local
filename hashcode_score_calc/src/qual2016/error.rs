use super::{Row, Col, DroneID, WarehouseID, ProductID, OrderID};
use crate::ScoringError;

use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum Qual2016ScoringError {
    #[error("Tried to access location row: {row}, col: {col} which is out of bounds")]
    LocationOutOfMap { row: Row, col: Col },
    #[error("There is a command for drone {drone_id}, which is not present in this case")]
    CommandIssuedToUnknownDrone { drone_id: DroneID },
    #[error("Drone {drone_id} carry too much")]
    DronePassedWeightLimit { drone_id: DroneID },
    #[error("You're trying to take too much from warehouse number {warehouse_id}")]
    OverTakingWarehouse { warehouse_id: WarehouseID },
    #[error("You're trying to unload more then you have of product {product_id} from drone number {drone_id}")]
    OverTakingDrone { drone_id: DroneID, product_id: ProductID },
    #[error("Trying to fly to an unknown warehouse {warehouse_id}")]
    UnknownWarehouse { warehouse_id: WarehouseID },
    #[error("Trying to supply to unknown order {order_id}")]
    UnknownOrder { order_id: OrderID },
    #[error("Trying to use unknown product {product_id}")]
    UnknownProduct { product_id: ProductID },
    #[error("You're over supplying order {order_id}")]
    OverSupplyingOrder { order_id: OrderID },
    #[error("Internal Error: drone {drone_id} didn't fly to warehouse {warehouse_id}")]
    DroneNotInWarehouse { drone_id: DroneID, warehouse_id: WarehouseID },
    #[error("Internal Error: drone {drone_id} didn't fly to order {order_id}")]
    DroneNotInOrderLocation{ drone_id: DroneID, order_id: OrderID },
    #[error("Loud, Deliver and Unload should have a positive number of items, drone {drone_id}")]
    CommandWithAmountZero {drone_id: DroneID},
}


impl From<Qual2016ScoringError> for ScoringError {
    fn from(e: Qual2016ScoringError) -> Self {
        ScoringError::ChallengeSpecific(Box::new(e))
    }
}
