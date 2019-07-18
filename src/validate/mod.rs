use crate::err::HecateError;

pub fn point(point: &String) -> Result<(f64, f64), HecateError> {
    let lnglat = point.split(",").collect::<Vec<&str>>();

    if lnglat.len() != 2 {
        return Err(HecateError::new(400, String::from("Point must be Lng,Lat"), None));
    }

    let lng: f64 = match lnglat[0].parse() {
        Ok(lng) => lng,
        _ => { return Err(HecateError::new(400, String::from("Longitude coordinate must be numeric"), None)); }
    };
    let lat: f64 = match lnglat[1].parse() {
        Ok(lat) => lat,
        _ => { return Err(HecateError::new(400, String::from("Latitude coordinate must be numeric"), None)); }
    };

    if lng < -180.0 || lng > 180.0 {
        return Err(HecateError::new(400, String::from("Longitude exceeds bounds"), None));
    } else if lat < -90.0 || lat > 90.0 {
        return Err(HecateError::new(400, String::from("Latitude exceeds bounds"), None));
    }

    Ok((lng, lat))
}

pub fn bbox(bbox: &Vec<f64>) -> Result<(), HecateError> {
    if bbox.len() != 4 {
        return Err(HecateError::new(400, String::from("Invalid BBOX"), None));
    }

    if bbox[0].is_nan() || bbox[0] < -180.0 || bbox[0] > 180.0 {
        return Err(HecateError::new(400, String::from("BBOX minX value must be a number between -180 and 180"), None));
    } else if bbox[1].is_nan() || bbox[1] < -90.0 || bbox[1] > 90.0 {
        return Err(HecateError::new(400, String::from("BBOX minY value must be a number between -90 and 90"), None));
    } else if bbox[2].is_nan() || bbox[2] < -180.0 || bbox[2] > 180.0 {
        return Err(HecateError::new(400, String::from("BBOX maxX value must be a number between -180 and 180"), None));
    } else if bbox[3].is_nan() || bbox[3] < -90.0 || bbox[3] > 90.0 {
        return Err(HecateError::new(400, String::from("BBOX maxY value must be a number between -90 and 90"), None));
    } else if bbox[0] > bbox[2] {
        return Err(HecateError::new(400, String::from("BBOX minX value cannot be greater than maxX value"), None));
    } else if bbox[1] > bbox[3] {
        return Err(HecateError::new(400, String::from("BBOX minY value cannot be greater than maxY value"), None));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]   
    fn valid_point() {
        assert_eq!(point(&String::from("-34.696,70.56")).ok(),Some((-34.696,70.56)), "ok - point coordinates is valid.");
    }

    #[test]
    fn invalid_point_longitude() {
        match point(&String::from("lng,70.56")){
            Ok(_) => (),
            Err(err) =>  assert_eq!(err.as_json(),json!({
                "code": 400,
                "reason": "Longitude coordinate must be numeric",
                "status": "Bad Request"
            }), "not ok - the longitude is invalid."),
        }
    }

    #[test]   
    fn valid_bbox() {
        assert_eq!(bbox(&[-107.578125,-30.600094,56.162109,46.377254].to_vec()).ok(),Some(()), "ok - point coordinates is valid.");
    }

    #[test]
    fn invalid_bbox_minX() {
        match bbox(&[-181.0,-30.600094,56.162109,46.377254].to_vec()){
            Ok(_) => (),
            Err(err) =>  assert_eq!(err.as_json(),json!({
                "code": 400,
                "reason": "BBOX minX value must be a number between -180 and 180",
                "status": "Bad Request"
            }), "not ok - the minX value is invalid."),
        }
    }
}


