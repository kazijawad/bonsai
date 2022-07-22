#ifndef SPHERE_H
#define SPHERE_H

#include <cmath>

#include "hittable.h"
#include "ray.h"
#include "utils.h"

class sphere : public hittable {
public:
    sphere() {}
    sphere(vec3 c, double r, std::shared_ptr<material> m) : center(c), radius(r), mat(m) {};

    virtual bool hit(const ray& r, double t0, double t1, hit_record& hitting) const override {
        auto oc = r.origin() - center;
        auto a = r.direction().length_squared();
        auto half_b = dot(oc, r.direction());
        auto c = oc.length_squared() - radius * radius;

        auto discriminant = half_b * half_b - a * c;
        if (discriminant < 0) return false;
        auto sqrtd = sqrt(discriminant);

        auto root = (-half_b - sqrtd) / a;
        if (root < t0 || root > t1) {
            root = (-half_b + sqrtd) / a;
            if (root < t0 || root > t1) return false;
        }

        hitting.t = root;
        hitting.p = r.at(hitting.t);
        auto outward_normal = (hitting.p - center) / radius;
        hitting.set_face_normal(r, outward_normal);
        get_sphere_uv(outward_normal, hitting.u, hitting.v);
        hitting.mat = mat;

        return true;
    }

    virtual bool bounding_box(double t0, double t1, aabb& bbox) const override {
        bbox = aabb(
            center - vec3(radius, radius, radius),
            center + vec3(radius, radius, radius)
        );
        return true;
    }

    virtual double pdf_value(const vec3& origin, const vec3& direction) const override {
        hit_record hitting;
        if (!this->hit(ray(origin, direction), 0.001, infinity, hitting)) {
            return 0;
        }

        auto cos_theta_max = sqrt(1 - radius * radius / (center - origin).length_squared());
        auto solid_angle = 2 * pi * (1 - cos_theta_max);

        return 1 / solid_angle;
    }

    virtual vec3 random(const vec3& origin) const override {
        vec3 direction = center - origin;
        auto distance_squared = direction.length_squared();
        auto uvw = onb();
        uvw.build_from_w(direction);
        return uvw.local(random_to_sphere(radius, distance_squared));
    }

private:
    vec3 center;
    double radius;
    std::shared_ptr<material> mat;

    static void get_sphere_uv(const vec3& p, double& u, double& v) {
        auto theta = acos(-p.y());
        auto phi = atan2(-p.z(), p.x()) + pi;
        u = phi / (2 * pi);
        v = theta / pi;
    }
};

#endif
