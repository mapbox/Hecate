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
    fn invalid_point_incomplete() {
        match point(&String::from("70.56")){
            Ok(_) => (),
            Err(err) =>  assert_eq!(err.as_json(),json!({
                "code": 400,
                "reason": "Point must be Lng,Lat",
                "status": "Bad Request"
            }), "not ok - the longitude is incomplete."),
        }
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
    fn invalid_point_latitude() {
        match point(&String::from("-122.44578,none.986")){
            Ok(_) => (),
            Err(err) =>  assert_eq!(err.as_json(),json!({
                "code": 400,
                "reason": "Latitude coordinate must be numeric",
                "status": "Bad Request"
            }), "not ok - the latitude is invalid."),
        }
    }

    #[test]
    fn invalid_point_longitude_out() {
        match point(&String::from("-181.578125,-30.600094")){
            Ok(_) => (),
            Err(err) =>  assert_eq!(err.as_json(),json!({
                "code": 400,
                "reason": "Longitude exceeds bounds",
                "status": "Bad Request"
            }), "not ok - the longitud is out of range."),
        }
    }

    #[test]
    fn invalid_point_latitude_out() {
        match point(&String::from("-107.578125,-100.600094")){
            Ok(_) => (),
            Err(err) =>  assert_eq!(err.as_json(),json!({
                "code": 400,
                "reason": "Latitude exceeds bounds",
                "status": "Bad Request"
            }), "not ok - the latitude is out of range."),
        }
    }

    #[test]   
    fn valid_bbox() {
        assert_eq!(bbox(&[-107.578125,-30.600094,56.162109,46.377254].to_vec()).ok(),Some(()), "ok - point coordinates is valid.");
    }

    #[test]
    fn invalid_bbox_incomplete() {
        match bbox(&[-181.0,-30.600094,56.162109].to_vec()){
            Ok(_) => (),
            Err(err) =>  assert_eq!(err.as_json(),json!({
                "code": 400,
                "reason": "Invalid BBOX",
                "status": "Bad Request"
            }), "not ok - the BBOX is incomplete."),
        }
    }

    #[test]
    fn invalid_bbox_minx() {
        match bbox(&[-181.0,-30.600094,56.162109,46.377254].to_vec()){
            Ok(_) => (),
            Err(err) =>  assert_eq!(err.as_json(),json!({
                "code": 400,
                "reason": "BBOX minX value must be a number between -180 and 180",
                "status": "Bad Request"
            }), "not ok - the minX value is invalid."),
        }
    }

    #[test]
    fn invalid_bbox_miny() {
        match bbox(&[-107.578125,-94.600094,56.162109,46.377254].to_vec()){
            Ok(_) => (),
            Err(err) =>  assert_eq!(err.as_json(),json!({
                "code": 400,
                "reason": "BBOX minY value must be a number between -90 and 90",
                "status": "Bad Request"
            }), "not ok - the minY value is invalid."),
        }
    }

    #[test]
    fn invalid_bbox_maxx() {
        match bbox(&[-107.578125,-30.600094,190.162109,46.377254].to_vec()){
            Ok(_) => (),
            Err(err) =>  assert_eq!(err.as_json(),json!({
                "code": 400,
                "reason": "BBOX maxX value must be a number between -180 and 180",
                "status": "Bad Request"
            }), "not ok - the maxX value is invalid."),
        }
    }

    #[test]
    fn invalid_bbox_maxy() {
        match bbox(&[-107.578125,-30.600094,56.162109,196.377254].to_vec()){
            Ok(_) => (),
            Err(err) =>  assert_eq!(err.as_json(),json!({
                "code": 400,
                "reason": "BBOX maxY value must be a number between -90 and 90",
                "status": "Bad Request"
            }), "not ok - the maxY value is invalid."),
        }
    }

    #[test]
    fn invalid_bbox_minx_gt_maxx() {
        match bbox(&[107.578125,-30.600094,56.162109,46.377254].to_vec()){
            Ok(_) => (),
            Err(err) =>  assert_eq!(err.as_json(),json!({
                "code": 400,
                "reason": "BBOX minX value cannot be greater than maxX value",
                "status": "Bad Request"
            }), "not ok - the minX value is greater than maxX."),
        }
    }

    #[test]
    fn invalid_bbox_miny_gt_maxy() {
        match bbox(&[-107.578125,30.600094,56.162109,-46.377254].to_vec()){
            Ok(_) => (),
            Err(err) =>  assert_eq!(err.as_json(),json!({
                "code": 400,
                "reason": "BBOX minY value cannot be greater than maxY value",
                "status": "Bad Request"
            }), "not ok - the minY value is greater than maxY."),
        }
    }
}
