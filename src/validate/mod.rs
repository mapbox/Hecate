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
