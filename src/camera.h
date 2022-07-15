#ifndef camera_h
#define camera_h

#include <cmath>

#include "vec3.h"
#include "ray.h"
#include "utils.h"

class camera {
public:
    camera(
        vec3 lookfrom,
        vec3 lookat,
        vec3 vup,
        double vfov,
        double aspect_ratio,
        double aperature,
        double focus_dist,
        double time0 = 0,
        double time1 = 0
    ) {
        auto theta = degrees_to_radians(vfov);
        auto h = tan(theta / 2);
        auto viewport_height = 2.0 * h;
        auto viewport_width = aspect_ratio * viewport_height;

        _w = unit_vector(lookfrom - lookat);
        _u = unit_vector(cross(vup, _w));
        _v = cross(_w, _u);

        _origin = lookfrom;
        _horizontal = focus_dist * viewport_width * _u;
        _vertical = focus_dist * viewport_height * _v;
        _lower_left_corner = _origin - _horizontal / 2 - _vertical / 2 - focus_dist * _w;

        _lens_radius = aperature / 2;
        _time0 = time0;
        _time1 = time1;
    }

    ray get_ray(double s, double t) const {
        vec3 rd = _lens_radius * random_in_unit_disk();
        vec3 offset = _u * rd.x() + _v * rd.y();
        return ray(
            _origin + offset,
            _lower_left_corner + s * _horizontal + t * _vertical - _origin - offset,
            random_double(_time0, _time1)
        );
    }

private:
    vec3 _origin;
    vec3 _lower_left_corner;
    vec3 _horizontal;
    vec3 _vertical;
    vec3 _u, _v, _w;
    double _lens_radius;
    double _time0, _time1;
};

#endif
