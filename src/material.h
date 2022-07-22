#ifndef material_h
#define material_h

#include <memory>

#include "ray.h"
#include "vec3.h"
#include "utils.h"
#include "texture.h"
#include "onb.h"
#include "pdf.h"

struct hit_record;

struct scatter_record {
    ray specular;
    bool is_specular;
    vec3 attenuation;
    std::shared_ptr<pdf> distribution;
};

class material {
public:
    virtual vec3 emitted(
        const ray& r,
        const hit_record& hit,
        double u,
        double v,
        const vec3& p
    ) const {
        return vec3();
    }

    virtual bool scatter(
        const ray& r,
        const hit_record& hit,
        scatter_record& scattering
    ) const {
        return false;
    }

    virtual double scattering_pdf(
        const ray& r,
        const hit_record& hit,
        const ray& scattered
    ) const {
        return 0;
    }
};

class lambertian : public material {
public:
    lambertian(const vec3& color) : map(std::make_shared<solid_color>(color)) {}
    lambertian(std::shared_ptr<texture> tex) : map(tex) {}

    virtual bool scatter(
        const ray& r,
        const hit_record& hit,
        scatter_record& scattering
    ) const override {
        scattering.is_specular = false;
        scattering.attenuation = map->value(hit.u, hit.v, hit.p);
        scattering.distribution = std::make_shared<cosine_pdf>(hit.normal);
        return true;
    }

    virtual double scattering_pdf(
        const ray& r,
        const hit_record& hit,
        const ray& scattered
    ) const override {
        auto cosine = dot(hit.normal, unit_vector(scattered.direction()));
        return cosine < 0 ? 0 : cosine / pi;
    }

private:
    std::shared_ptr<texture> map;
};

class metal : public material {
public:
    metal(const vec3& c, double f) : color(c), fuzz(f < 1 ? f : 1) {}

    virtual bool scatter(
        const ray& r,
        const hit_record& hit,
        scatter_record& scattering
    ) const override {
        vec3 reflected = reflect(unit_vector(r.direction()), hit.normal);
        scattering.specular = ray(hit.p, reflected + fuzz * random_in_unit_sphere());
        scattering.attenuation = color;
        scattering.is_specular = true;
        scattering.distribution = 0;
        return true;
    }

private:
    vec3 color;
    double fuzz;
};

class dielectric : public material {
public:
    dielectric(double index_of_refraction) : ior(index_of_refraction) {}

    virtual bool scatter(
        const ray& r,
        const hit_record& hit,
        scatter_record& scattering
    ) const override {
        scattering.is_specular = true;
        scattering.distribution = nullptr;
        scattering.attenuation = vec3(1.0);

        double refraction_ratio = hit.front_face ? (1.0 / ior) : ior;
        vec3 unit_direction = unit_vector(r.direction());

        double cos_theta = fmin(dot(-unit_direction, hit.normal), 1.0);
        double sin_theta = sqrt(1.0 - cos_theta * cos_theta);

        vec3 direction;
        bool cannot_refract = refraction_ratio * sin_theta > 1.0;
        if (cannot_refract || reflectance(cos_theta, refraction_ratio) > random_double()) {
            direction = reflect(unit_direction, hit.normal);
        } else {
            direction = refract(unit_direction, hit.normal, refraction_ratio);
        }

        scattering.specular = ray(hit.p, direction, r.time());

        return true;
    }

private:
    double ior;

    static double reflectance(double cosine, double ref_idx) {
        // Use Schlick's approximation for reflectance.
        auto r0 = (1 - ref_idx) / (1 + ref_idx);
        r0 = r0 * r0;
        return r0 + (1 - r0) * pow((1 - cosine), 5);
    }
};

class diffuse_light : public material {
public:
    diffuse_light(vec3 color) : map(std::make_shared<solid_color>(color)) {}
    diffuse_light(std::shared_ptr<texture> tex) : map(tex) {}

    virtual vec3 emitted(
        const ray& r,
        const hit_record& hit,
        double u,
        double v,
        const vec3& p
    ) const override {
        if (!hit.front_face) {
            return vec3();
        }
        return map->value(u, v, p);
    }

private:
    std::shared_ptr<texture> map;
};

class isotropic : public material {
public:
    isotropic(vec3 color) : map(std::make_shared<solid_color>(color)) {}
    isotropic(std::shared_ptr<texture> tex) : map(tex) {}

    virtual bool scatter(
        const ray& r,
        const hit_record& hit,
        scatter_record& scattering
    ) const override {
        scattering.specular = ray(hit.p, random_in_unit_sphere(), r.time());
        scattering.attenuation = map->value(hit.u, hit.v, hit.p);
        scattering.is_specular = false;
        scattering.distribution = nullptr;
        return true;
    }

private:
    std::shared_ptr<texture> map;
};

#endif
