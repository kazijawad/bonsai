#ifndef aarect_h
#define aarect_h

#include <memory>

#include "material.h"

class xyrect : public hittable {
public:
    xyrect() {}
    xyrect(
        double x0, double x1, double y0, double y1, double k, std::shared_ptr<material> mat
    ) : _x0(x0), _x1(x1), _y0(y0), _y1(y1), _k(k), _mat(mat) {}

    virtual bool hit(const ray& r, double t_min, double t_max, hit_record& record) const override {
        auto t = (_k - r.origin().z()) / r.direction().z();
        if (t < t_min || t > t_max) return false;

        auto x = r.origin().x() + t * r.direction().x();
        auto y = r.origin().y() + t * r.direction().y();
        if (x < _x0 || x > _x1 || y < _y0 || y > _y1) return false;

        record.u = (x - _x0) / (_x1 - _x0);
        record.v = (y - _y0) / (_y1 - _y0);
        record.t = t;

        record.set_face_normal(r, vec3(0, 0, 1));
        record.mat = _mat;
        record.p = r.at(t);

        return true;
    }

    virtual bool bounding_box(double t0, double t1, aabb& output_box) const override {
        output_box = aabb(vec3(_x0, _y0, _k - 0.0001), vec3(_x1, _y1, _k + 0.0001));
        return true;
    }

private:
    std::shared_ptr<material> _mat;
    double _x0, _x1, _y0, _y1, _k;
};

class xzrect : public hittable {
public:
    xzrect() {}
    xzrect(
        double x0, double x1, double z0, double z1, double k, std::shared_ptr<material> mat
    ) : _x0(x0), _x1(x1), _z0(z0), _z1(z1), _k(k), _mat(mat) {}

    virtual bool hit(const ray& r, double t_min, double t_max, hit_record& record) const override {
        auto t = (_k - r.origin().y()) / r.direction().y();
        if (t < t_min || t > t_max) return false;

        auto x = r.origin().x() + t * r.direction().x();
        auto z = r.origin().z() + t * r.direction().z();
        if (x < _x0 || x > _x1 || z < _z0 || z > _z1) return false;

        record.u = (x - _x0) / (_x1 - _x0);
        record.v = (z - _z0) / (_z1 - _z0);
        record.t = t;

        record.set_face_normal(r, vec3(0, 1, 0));
        record.mat = _mat;
        record.p = r.at(t);

        return true;
    }

    virtual bool bounding_box(double t0, double t1, aabb& output_box) const override {
        output_box = aabb(vec3(_x0, _k - 0.0001, _z0), vec3(_x1, _k + 0.0001, _z1));
        return true;
    }

private:
    std::shared_ptr<material> _mat;
    double _x0, _x1, _z0, _z1, _k;
};

class yzrect : public hittable {
public:
    yzrect() {}
    yzrect(
        double y0, double y1, double z0, double z1, double k, std::shared_ptr<material> mat
    ) : _y0(y0), _y1(y1), _z0(z0), _z1(z1), _k(k), _mat(mat) {}

    virtual bool hit(const ray& r, double t_min, double t_max, hit_record& record) const override {
        auto t = (_k - r.origin().x()) / r.direction().x();
        if (t < t_min || t > t_max) return false;

        auto y = r.origin().y() + t * r.direction().y();
        auto z = r.origin().z() + t * r.direction().z();
        if (y < _y0 || y > _y1 || z < _z0 || z > _z1) return false;

        record.u = (y - _y0) / (_y1 - _y0);
        record.v = (z - _z0) / (_z1 - _z0);
        record.t = t;

        record.set_face_normal(r, vec3(0, 1, 0));
        record.mat = _mat;
        record.p = r.at(t);

        return true;
    }

    virtual bool bounding_box(double t0, double t1, aabb& output_box) const override {
        output_box = aabb(vec3(_k - 0.0001, _y0, _z0), vec3(_k + 0.0001, _y1, _z1));
        return true;
    }

private:
    std::shared_ptr<material> _mat;
    double _y0, _y1, _z0, _z1, _k;
};

#endif
