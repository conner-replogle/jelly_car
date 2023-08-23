use bevy::prelude::*;

use crate::soft_body::{MassPoint, Spring};



pub fn bounding_box(points : &Vec<MassPoint>) -> Rect{
    let points: Vec<(f32, f32)> = points.iter().map(|a| (a.transform.x,a.transform.y)).collect();
    
    //get the min and max x and y from points
    let mut x0 = points[0].0;
    let mut y0 = points[0].1;
    let mut x1 = points[0].0;
    let mut y1 = points[0].1;

    for point in points.iter(){
        if point.0 < x0{
            x0 = point.0;
        }
        if point.0 > x1{
            x1 = point.0;
        }
        if point.1 < y0{
            y0 = point.1;
        }
        if point.1 > y1{
            y1 = point.1;
        }
    }
    
    Rect::new(x0, y0, x1, y1)


}
//function to find the nearest point on a line to a point
pub fn closest_point_on_line(point: Vec2, line: (Vec2,Vec2)) -> Vec2{
    //find the nearest point on the line to the point
    //https://stackoverflow.com/questions/3120357/get-closest-point-to-a-line
    let a_to_p = point - line.0;
    let a_to_b = line.1 - line.0;

    let a_to_b_squared = a_to_b.dot(a_to_b);
    let a_to_p_dot_a_to_b = a_to_p.dot(a_to_b);
    let mut t = a_to_p_dot_a_to_b / a_to_b_squared;

    if t < 0.0 {
        t = 0.0;
    } else if t > 1.0 {
        t = 1.0;
    }

    let nearest = line.0 + a_to_b * t;
    //return nearest point on line
    return nearest
}


//find the point line and point on that line from a point
pub fn count_line_intersections_horizontal(lines: &Vec<(Vec2,Vec2,(Entity,Entity))>,segment: (Vec2,Vec2),point: Vec2,gizmo:&mut Gizmos) -> (u32,Vec2,(Vec2,Vec2,(Entity,Entity))){
    let mut intersections = 0;
    //count how many times this segment intersects with the segments from self.lines()
    let mut nearest_point= Vec2::MAX;
    let mut nearest_line = lines[0];
    let mut nearest_distance = f32::MAX;
    for line in lines{
        let closest = closest_point_on_line(point, (line.0,line.1));
        let distance = point.distance(closest);
        if distance < nearest_distance{
            nearest_distance = distance;
            nearest_point = closest;
            nearest_line = line.clone();
        }
        //println!("Line: {:?}",line);
        let (p0,p1) = (line.0,line.1);
        let (p2,p3) = segment;
        //println!("P0: {:?} P1: {:?} P2: {:?} P3: {:?}",p0,p1,p2,p3);
        let s1_x = p1.x - p0.x;
        let s1_y = p1.y - p0.y;
        let s2_x = p3.x - p2.x;
        let s2_y = p3.y - p2.y;
        let s = (-s1_y * (p0.x - p2.x) + s1_x * (p0.y - p2.y)) / (-s2_x * s1_y + s1_x * s2_y);
        let t = ( s2_x * (p0.y - p2.y) - s2_y * (p0.x - p2.x)) / (-s2_x * s1_y + s1_x * s2_y);
        //println!("S: {:?} T: {:?}",s,t);
        if s >= 0.0 && s <= 1.0 && t >= 0.0 && t <= 1.0{
            // Collision detected
            let intersection_x = p0.x + (t * s1_x);
            let intersection_y = p0.y + (t * s1_y);
            //println!("Intersect: {:?} {:?}",intersection_x,intersection_y);
            intersections += 1;
            gizmo.circle(Vec3::new(intersection_x,intersection_y,0.),Vec3::Z,3., Color::RED);
        }
    }

    
    (    
        intersections,
        nearest_point,
        nearest_line
    )
}

pub fn lines(mass_points: &Query<&mut MassPoint>,springs: Vec<&Spring>) -> Vec< (Vec2,Vec2,(Entity,Entity))>{
    //TODO fix this
    let mut lines = Vec::new();
    for spring in springs.iter(){
        let from = mass_points.get(spring.from).unwrap();
        let to = mass_points.get(spring.to).unwrap();

        lines.push((from.transform,to.transform,(spring.from,spring.to)));
    }
    lines
}
