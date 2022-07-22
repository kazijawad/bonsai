#ifndef hittable_list_h
#define hittable_list_h

#include <vector>

#include "hittable.h"
#include "ray.h"
#include "aabb.h"

class hittable_list : public hittable {
public:
    std::vector<std::shared_ptr<hittable>> objects;

    hittable_list() {}
    hittable_list(std::shared_ptr<hittable> object) {
        add(object);
    }

    void add(std::shared_ptr<hittable> object) {
        objects.push_back(object);
    }

    void clear() {
        objects.clear();
    }

    virtual bool hit(const ray& r, double t_min, double t_max, hit_record& hitting) const override {
        hit_record temp;
        bool hit_anything = false;
        auto closest_so_far = t_max;

        for (const auto& object : objects) {
            if (object->hit(r, t_min, closest_so_far, temp)) {
                hit_anything = true;
                closest_so_far = temp.t;
                hitting = temp;
            }
        }

        return hit_anything;
    }

    virtual bool bounding_box(double t0, double t1, aabb& bbox) const override {
        if (objects.empty()) return false;

        aabb temp;
        bool first_box = true;

        for (const auto& object : objects) {
            if (!object->bounding_box(t0, t1, temp)) {
                return false;
            }
            bbox = first_box ? temp : aabb::surrounding_box(bbox, temp);
            first_box = false;
        }

        return true;
    }

    virtual double pdf_value(const vec3& origin, const vec3& direction) const override {
        auto weight = 1.0 / objects.size();
        auto sum = 0.0;
        for (const auto& object : objects) {
            sum += weight * object->pdf_value(origin, direction);
        }
        return sum;
    }

    virtual vec3 random(const vec3& origin) const override {
        auto int_size = static_cast<int>(objects.size());
        return objects[random_int(0, int_size - 1)]->random(origin);
    }
};

#endif
