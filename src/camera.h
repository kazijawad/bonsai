#ifndef camera_h
#define camera_h

#include <cmath>

#include "vec3.h"
#include "ray.h"
#include "utils.h"

class camera {
public:
    camera(
        vec3 position,
        vec3 look_at,
        double fov,
        double aspect_ratio,
        double aperature,
        double focus_distance,
        double time0 = 0,
        double time1 = 0
    ) {
        auto theta = degrees_to_radians(fov);
        auto h = tan(theta / 2);
        auto viewport_height = 2.0 * h;
        auto viewport_width = aspect_ratio * viewport_height;

        w = unit_vector(position - look_at);
        u = unit_vector(cross(up, w));
        v = cross(w, u);

        origin = position;
        horizontal = focus_distance * viewport_width * u;
        vertical = focus_distance * viewport_height * v;
        lower_left_corner = origin - horizontal / 2 - vertical / 2 - focus_distance * w;

        lens_radius = aperature / 2;
        time0 = time0;
        time1 = time1;
    }

    ray get_ray(double s, double t) const {
        vec3 rd = lens_radius * random_in_unit_disk();
        vec3 offset = u * rd.x() + v * rd.y();
        return ray(
            origin + offset,
            lower_left_corner + s * horizontal + t * vertical - origin - offset,
            random_double(time0, time1)
        );
    }

private:
    vec3 up = vec3(0, 1, 0);
    vec3 origin;
    vec3 lower_left_corner;
    vec3 horizontal;
    vec3 vertical;
    vec3 u, v, w;
    double lens_radius;
    double time0, time1;
};

#endif
