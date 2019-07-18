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

pub fn bbox(bbox: &Vec<f64>) -> Result<Vec<f64>, HecateError> {
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

    Ok(bbox.to_vec())
}

