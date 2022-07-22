#ifndef pdf_h
#define pdf_h

#include "vec3.h"
#include "onb.h"

class pdf {
public:
    virtual ~pdf() {}

    virtual double value(const vec3& direction) const = 0;
    virtual vec3 generate() const = 0;
};

class cosine_pdf : public pdf {
public:
    cosine_pdf(const vec3& w) {
        uvw.build_from_w(w);
    }

    virtual double value(const vec3& direction) const override {
        auto cosine = dot(unit_vector(direction), uvw.w());
        return (cosine <= 0) ? 0 : cosine / pi;
    }

    virtual vec3 generate() const override {
        return uvw.local(random_cosine_direction());
    }

private:
    onb uvw;
};

class hittable_pdf : public pdf {
public:
    hittable_pdf(std::shared_ptr<hittable> p, const vec3& o) : ref(p), origin(o) {}

    virtual double value(const vec3& direction) const override {
        return ref->pdf_value(origin, direction);
    }

    virtual vec3 generate() const override {
        return ref->random(origin);
    }

private:
    vec3 origin;
    std::shared_ptr<hittable> ref;
};

class mixture_pdf : public pdf {
public:
    mixture_pdf(std::shared_ptr<pdf> p0, std::shared_ptr<pdf> p1) {
        pdfs[0] = p0;
        pdfs[1] = p1;
    }

    virtual double value(const vec3& direction) const override {
        return 0.5 * pdfs[0]->value(direction) + 0.5 * pdfs[1]->value(direction);
    }

    virtual vec3 generate() const override {
        if (random_double() < 0.5) {
            return pdfs[0]->generate();
        }
        return pdfs[1]->generate();
    }

private:
    std::shared_ptr<pdf> pdfs[2];
};

#endif
