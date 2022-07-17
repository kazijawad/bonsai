#ifndef aarect_h
#define aarect_h

#include <memory>

#include "material.h"

class xyrect : public hittable {
public:
    xyrect() {}
    xyrect(
        double x0, double x1, double y0, double y1, double k, std::shared_ptr<material> mat
    ) : x0(x0), x1(x1), y0(y0), y1(y1), k(k), mat(mat) {}

    virtual bool hit(const ray& r, double t_min, double t_max, hit_record& rec) const override {
        auto t = (k - r.origin().z()) / r.direction().z();
        if (t < t_min || t > t_max) return false;

        auto x = r.origin().x() + t * r.direction().x();
        auto y = r.origin().y() + t * r.direction().y();
        if (x < x0 || x > x1 || y < y0 || y > y1) return false;

        rec.u = (x - x0) / (x1 - x0);
        rec.v = (y - y0) / (y1 - y0);
        rec.t = t;

        rec.set_face_normal(r, vec3(0, 0, 1));
        rec.mat = mat;
        rec.p = r.at(t);

        return true;
    }

    virtual bool bounding_box(double t0, double t1, aabb& output_box) const override {
        output_box = aabb(vec3(x0, y0, k - 0.0001), vec3(x1, y1, k + 0.0001));
        return true;
    }

private:
    std::shared_ptr<material> mat;
    double x0, x1, y0, y1, k;
};

class xzrect : public hittable {
public:
    xzrect() {}
    xzrect(
        double x0, double x1, double z0, double z1, double k, std::shared_ptr<material> mat
    ) : x0(x0), x1(x1), z0(z0), z1(z1), k(k), mat(mat) {}

    virtual bool hit(const ray& r, double t_min, double t_max, hit_record& rec) const override {
        auto t = (k - r.origin().y()) / r.direction().y();
        if (t < t_min || t > t_max) return false;

        auto x = r.origin().x() + t * r.direction().x();
        auto z = r.origin().z() + t * r.direction().z();
        if (x < x0 || x > x1 || z < z0 || z > z1) return false;

        rec.u = (x - x0) / (x1 - x0);
        rec.v = (z - z0) / (z1 - z0);
        rec.t = t;

        rec.set_face_normal(r, vec3(0, 1, 0));
        rec.mat = mat;
        rec.p = r.at(t);

        return true;
    }

    virtual bool bounding_box(double t0, double t1, aabb& output_box) const override {
        output_box = aabb(vec3(x0, k - 0.0001, z0), vec3(x1, k + 0.0001, z1));
        return true;
    }

private:
    std::shared_ptr<material> mat;
    double x0, x1, z0, z1, k;
};

class yzrect : public hittable {
public:
    yzrect() {}
    yzrect(
        double y0, double y1, double z0, double z1, double k, std::shared_ptr<material> mat
    ) : y0(y0), y1(y1), z0(z0), z1(z1), k(k), mat(mat) {}

    virtual bool hit(const ray& r, double t_min, double t_max, hit_record& rec) const override {
        auto t = (k - r.origin().x()) / r.direction().x();
        if (t < t_min || t > t_max) return false;

        auto y = r.origin().y() + t * r.direction().y();
        auto z = r.origin().z() + t * r.direction().z();
        if (y < y0 || y > y1 || z < z0 || z > z1) return false;

        rec.u = (y - y0) / (y1 - y0);
        rec.v = (z - z0) / (z1 - z0);
        rec.t = t;

        rec.set_face_normal(r, vec3(1, 0, 0));
        rec.mat = mat;
        rec.p = r.at(t);

        return true;
    }

    virtual bool bounding_box(double t0, double t1, aabb& output_box) const override {
        output_box = aabb(vec3(k - 0.0001, y0, z0), vec3(k + 0.0001, y1, z1));
        return true;
    }

private:
    std::shared_ptr<material> mat;
    double y0, y1, z0, z1, k;
};

#endif
