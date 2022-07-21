#ifndef material_h
#define material_h

#include <memory>

#include "ray.h"
#include "vec3.h"
#include "utils.h"
#include "texture.h"
#include "onb.h"

struct hit_record;

class material {
public:
    virtual bool scatter(const ray& r_in, const hit_record& rec, vec3& attenuation, ray& scattered) const {
        return false;
    }

    virtual bool scatter(const ray& r, const hit_record& rec, vec3& color, ray& scattered, double& pdf) const {
        return false;
    }

    virtual double scattering_pdf(const ray& r, const hit_record& rec, const ray& scattered) const {
        return 0;
    }

    virtual vec3 emitted(double u, double v, const vec3& p) const {
        return vec3();
    }
};

class lambertian : public material {
public:
    lambertian(const vec3& a) : albedo(std::make_shared<solid_color>(a)) {}
    lambertian(std::shared_ptr<texture> a) : albedo(a) {}

    virtual bool scatter(const ray& r, const hit_record& rec, vec3& color, ray& scattered, double& pdf) const override {
        onb uvw;
        uvw.build_from_w(rec.normal);
        auto direction = uvw.local(random_cosine_direction());
        if (direction.near_zero()) {
            direction = rec.normal;
        }
        scattered = ray(rec.p, unit_vector(direction), r.time());
        color = albedo->value(rec.u, rec.v, rec.p);
        pdf = dot(uvw.w(), scattered.direction()) / pi;
        return true;
    }

    virtual double scattering_pdf(const ray& r, const hit_record& rec, const ray& scattered) const override {
        auto cosine = dot(rec.normal, unit_vector(scattered.direction()));
        return cosine < 0 ? 0 : cosine / pi;
    }

private:
    std::shared_ptr<texture> albedo;
};

class metal : public material {
public:
    vec3 albedo;
    double fuzz;

    metal(const vec3& a, double f) : albedo(a), fuzz(f < 1 ? f : 1) {}

    virtual bool scatter(const ray& r_in, const hit_record& rec, vec3& attenuation, ray& scattered) const override {
        vec3 reflected = reflect(unit_vector(r_in.direction()), rec.normal);
        scattered = ray(rec.p, reflected + fuzz * random_in_unit_sphere(), r_in.time());
        attenuation = albedo;
        return (dot(scattered.direction(), rec.normal) > 0);
    }
};

class dielectric : public material {
public:
    double ir;

    dielectric(double index_of_refraction) : ir(index_of_refraction) {}

    virtual bool scatter(const ray& r_in, const hit_record& rec, vec3& attenuation, ray& scattered) const override {
        attenuation = vec3(1.0, 1.0, 1.0);
        double refraction_ratio = rec.front_face ? (1.0 / ir) : ir;

        vec3 unit_direction = unit_vector(r_in.direction());
        double cos_theta = fmin(dot(-unit_direction, rec.normal), 1.0);
        double sin_theta = sqrt(1.0 - cos_theta * cos_theta);

        bool cannot_refract = refraction_ratio * sin_theta > 1.0;
        vec3 direction;
        if (cannot_refract || reflectance(cos_theta, refraction_ratio) > random_double()) {
            direction = reflect(unit_direction, rec.normal);
        } else {
            direction = refract(unit_direction, rec.normal, refraction_ratio);
        }

        scattered = ray(rec.p, direction, r_in.time());
        return true;
    }

private:
    static double reflectance(double cosine, double ref_idx) {
        // Use Schlick's approximation for reflectance.
        auto r0 = (1 - ref_idx) / (1 + ref_idx);
        r0 = r0 * r0;
        return r0 + (1 - r0) * pow((1 - cosine), 5);
    }
};

class diffuse_light : public material {
public:
    diffuse_light(std::shared_ptr<texture> tex) : emit(tex) {}
    diffuse_light(vec3 color) : emit(std::make_shared<solid_color>(color)) {}

    virtual bool scatter(const ray& r_in, const hit_record& rec, vec3& attenuation, ray& scattered) const override {
        return false;
    }

    virtual vec3 emitted(double u, double v, const vec3& p) const override {
        return emit->value(u, v, p);
    }

private:
    std::shared_ptr<texture> emit;
};

class isotropic : public material {
public:
    isotropic(vec3 c) : albedo(std::make_shared<solid_color>(c)) {}
    isotropic(std::shared_ptr<texture> tex) : albedo(tex) {}

    virtual bool scatter(const ray& r_in, const hit_record& rec, vec3& attenuation, ray& scattered) const override {
        scattered = ray(rec.p, random_in_unit_sphere(), r_in.time());
        attenuation = albedo->value(rec.u, rec.v, rec.p);
        return true;
    }

private:
    std::shared_ptr<texture> albedo;
};

#endif
