#ifndef box_h
#define box_h

#include <memory>

#include "aarect.h"
#include "hittable_list.h"

class box : public hittable {
public:
    box() {}
    box(const vec3& p0, const vec3& p1, std::shared_ptr<material> mat) {
        min = p0;
        max = p1;

        sides.add(std::make_shared<xyrect>(p0.x(), p1.x(), p0.y(), p1.y(), p1.z(), mat));
        sides.add(std::make_shared<xyrect>(p0.x(), p1.x(), p0.y(), p1.y(), p0.z(), mat));

        sides.add(std::make_shared<xzrect>(p0.x(), p1.x(), p0.z(), p1.z(), p1.y(), mat));
        sides.add(std::make_shared<xzrect>(p0.x(), p1.x(), p0.z(), p1.z(), p0.y(), mat));

        sides.add(std::make_shared<yzrect>(p0.y(), p1.y(), p0.z(), p1.z(), p1.x(), mat));
        sides.add(std::make_shared<yzrect>(p0.y(), p1.y(), p0.z(), p1.z(), p0.x(), mat));
    }

    virtual bool hit(const ray& r, double t_min, double t_max, hit_record& rec) const override {
        return sides.hit(r, t_min, t_max, rec);
    }

    virtual bool bounding_box(double time0, double time1, aabb& output_box) const override {
        output_box = aabb(min, max);
        return true;
    }

private:
    vec3 min;
    vec3 max;
    hittable_list sides;
};

#endif
