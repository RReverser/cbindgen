#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <cassert>

template<typename T>
struct StylePoint {
  T x;
  T y;
};

template<typename T>
union StyleFoo {
  enum class Tag : uint8_t {
    Foo,
    Bar,
    Baz,
    Bazz,
  };

  struct Foo_Body {
    Tag tag;
    int32_t x;
    StylePoint<T> y;
    StylePoint<float> z;
  };

  struct Bar_Body {
    Tag tag;
    T _0;
  };

  struct Baz_Body {
    Tag tag;
    StylePoint<T> _0;
  };

  struct {
    Tag tag;
  };
  Foo_Body foo;
  Bar_Body bar;
  Baz_Body baz;

  static StyleFoo Foo(const int32_t &aX,
                      const StylePoint<T> &aY,
                      const StylePoint<float> &aZ) {
    StyleFoo result;
    result.foo.x = aX;
    result.foo.y = aY;
    result.foo.z = aZ;
    result.tag = Tag::Foo;
    return result;
  }

  static StyleFoo Bar(const T &a0) {
    StyleFoo result;
    result.bar._0 = a0;
    result.tag = Tag::Bar;
    return result;
  }

  static StyleFoo Baz(const StylePoint<T> &a0) {
    StyleFoo result;
    result.baz._0 = a0;
    result.tag = Tag::Baz;
    return result;
  }

  static StyleFoo Bazz() {
    StyleFoo result;
    result.tag = Tag::Bazz;
    return result;
  }

  bool IsFoo() const {
    return tag == Tag::Foo;
  }

  bool IsBar() const {
    return tag == Tag::Bar;
  }

  bool IsBaz() const {
    return tag == Tag::Baz;
  }

  bool IsBazz() const {
    return tag == Tag::Bazz;
  }

  const Foo_Body& AsFoo() const {
    assert(IsFoo());
    return foo;
  }

  Foo_Body& AsFoo() {
    assert(IsFoo());
    return foo;
  }

  const T& AsBar() const {
    assert(IsBar());
    return bar._0;
  }

  T& AsBar() {
    assert(IsBar());
    return bar._0;
  }

  const StylePoint<T>& AsBaz() const {
    assert(IsBaz());
    return baz._0;
  }

  StylePoint<T>& AsBaz() {
    assert(IsBaz());
    return baz._0;
  }
};

template<typename T>
struct StyleBar {
  enum class Tag {
    Bar1,
    Bar2,
    Bar3,
    Bar4,
  };

  struct StyleBar1_Body {
    int32_t x;
    StylePoint<T> y;
    StylePoint<float> z;
  };

  struct StyleBar2_Body {
    T _0;
  };

  struct StyleBar3_Body {
    StylePoint<T> _0;
  };

  Tag tag;
  union {
    StyleBar1_Body bar1;
    StyleBar2_Body bar2;
    StyleBar3_Body bar3;
  };

  static StyleBar Bar1(const int32_t &aX,
                       const StylePoint<T> &aY,
                       const StylePoint<float> &aZ) {
    StyleBar result;
    result.bar1.x = aX;
    result.bar1.y = aY;
    result.bar1.z = aZ;
    result.tag = Tag::Bar1;
    return result;
  }

  static StyleBar Bar2(const T &a0) {
    StyleBar result;
    result.bar2._0 = a0;
    result.tag = Tag::Bar2;
    return result;
  }

  static StyleBar Bar3(const StylePoint<T> &a0) {
    StyleBar result;
    result.bar3._0 = a0;
    result.tag = Tag::Bar3;
    return result;
  }

  static StyleBar Bar4() {
    StyleBar result;
    result.tag = Tag::Bar4;
    return result;
  }

  bool IsBar1() const {
    return tag == Tag::Bar1;
  }

  bool IsBar2() const {
    return tag == Tag::Bar2;
  }

  bool IsBar3() const {
    return tag == Tag::Bar3;
  }

  bool IsBar4() const {
    return tag == Tag::Bar4;
  }

  const StyleBar1_Body& AsBar1() const {
    assert(IsBar1());
    return bar1;
  }

  StyleBar1_Body& AsBar1() {
    assert(IsBar1());
    return bar1;
  }

  const T& AsBar2() const {
    assert(IsBar2());
    return bar2._0;
  }

  T& AsBar2() {
    assert(IsBar2());
    return bar2._0;
  }

  const StylePoint<T>& AsBar3() const {
    assert(IsBar3());
    return bar3._0;
  }

  StylePoint<T>& AsBar3() {
    assert(IsBar3());
    return bar3._0;
  }
};

union StyleBaz {
  enum class Tag : uint8_t {
    Baz1,
    Baz2,
    Baz3,
  };

  struct Baz1_Body {
    Tag tag;
    StyleBar<uint32_t> _0;
  };

  struct Baz2_Body {
    Tag tag;
    StylePoint<int32_t> _0;
  };

  struct {
    Tag tag;
  };
  Baz1_Body baz1;
  Baz2_Body baz2;

  static StyleBaz Baz1(const StyleBar<uint32_t> &a0) {
    StyleBaz result;
    result.baz1._0 = a0;
    result.tag = Tag::Baz1;
    return result;
  }

  static StyleBaz Baz2(const StylePoint<int32_t> &a0) {
    StyleBaz result;
    result.baz2._0 = a0;
    result.tag = Tag::Baz2;
    return result;
  }

  static StyleBaz Baz3() {
    StyleBaz result;
    result.tag = Tag::Baz3;
    return result;
  }

  bool IsBaz1() const {
    return tag == Tag::Baz1;
  }

  bool IsBaz2() const {
    return tag == Tag::Baz2;
  }

  bool IsBaz3() const {
    return tag == Tag::Baz3;
  }

  const StyleBar<uint32_t>& AsBaz1() const {
    assert(IsBaz1());
    return baz1._0;
  }

  StyleBar<uint32_t>& AsBaz1() {
    assert(IsBaz1());
    return baz1._0;
  }

  const StylePoint<int32_t>& AsBaz2() const {
    assert(IsBaz2());
    return baz2._0;
  }

  StylePoint<int32_t>& AsBaz2() {
    assert(IsBaz2());
    return baz2._0;
  }
};

struct StyleTaz {
  enum class Tag : uint8_t {
    Taz1,
    Taz2,
    Taz3,
  };

  struct StyleTaz1_Body {
    StyleBar<uint32_t> _0;
  };

  struct StyleTaz2_Body {
    StyleBaz _0;
  };

  Tag tag;
  union {
    StyleTaz1_Body taz1;
    StyleTaz2_Body taz2;
  };

  static StyleTaz Taz1(const StyleBar<uint32_t> &a0) {
    StyleTaz result;
    result.taz1._0 = a0;
    result.tag = Tag::Taz1;
    return result;
  }

  static StyleTaz Taz2(const StyleBaz &a0) {
    StyleTaz result;
    result.taz2._0 = a0;
    result.tag = Tag::Taz2;
    return result;
  }

  static StyleTaz Taz3() {
    StyleTaz result;
    result.tag = Tag::Taz3;
    return result;
  }

  bool IsTaz1() const {
    return tag == Tag::Taz1;
  }

  bool IsTaz2() const {
    return tag == Tag::Taz2;
  }

  bool IsTaz3() const {
    return tag == Tag::Taz3;
  }

  const StyleBar<uint32_t>& AsTaz1() const {
    assert(IsTaz1());
    return taz1._0;
  }

  StyleBar<uint32_t>& AsTaz1() {
    assert(IsTaz1());
    return taz1._0;
  }

  const StyleBaz& AsTaz2() const {
    assert(IsTaz2());
    return taz2._0;
  }

  StyleBaz& AsTaz2() {
    assert(IsTaz2());
    return taz2._0;
  }
};

extern "C" {

void foo(const StyleFoo<int32_t> *foo,
         const StyleBar<int32_t> *bar,
         const StyleBaz *baz,
         const StyleTaz *taz);

} // extern "C"
