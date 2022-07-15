#ifndef ray_h
#define ray_h

#include "vec3.h"

class ray {
public:
    ray() {}
    ray(
        const vec3& origin, const vec3& direction, double time = 0.0
    ) : _origin(origin), _direction(direction), _time(time) {}

    vec3 origin() const {
        return _origin;
    }

    vec3 direction() const {
        return _direction;
    }

    double time() const {
        return _time;
    }

    vec3 at(double t) const {
        return _origin + t * _direction;
    }

private:
    vec3 _origin;
    vec3 _direction;
    double _time;
};

#endif
