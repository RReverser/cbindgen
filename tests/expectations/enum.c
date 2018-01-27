#include <stdint.h>
#include <stdlib.h>
#include <stdbool.h>

enum A {
  a1 = 0,
  a2 = 2,
  a3,
  a4 = 5,
};
typedef uint32_t A;

enum B {
  b1 = 0,
  b2 = 2,
  b3,
  b4 = 5,
};
typedef uint16_t B;

enum C {
  c1 = 0,
  c2 = 2,
  c3,
  c4 = 5,
};
typedef uint8_t C;

enum D {
  d1 = 0,
  d2 = 2,
  d3,
  d4 = 5,
};
typedef uintptr_t D;

enum E {
  e1 = 0,
  e2 = 2,
  e3,
  e4 = 5,
};
typedef intptr_t E;

enum F_Tag {
  Foo,
  Bar,
  Baz,
};
typedef uint8_t F_Tag;

typedef struct {
  F_Tag tag;
  int16_t _0;
} Foo_Body;

typedef struct {
  F_Tag tag;
  uint8_t x;
  int16_t y;
} Bar_Body;

typedef union {
  F_Tag tag;
  Foo_Body foo;
  Bar_Body bar;
} F;

typedef enum {
  G_Foo,
  G_Bar,
  G_Baz,
} G_Tag;

typedef struct {
  int16_t _0;
} G_Foo_Body;

typedef struct {
  uint8_t x;
  int16_t y;
} G_Bar_Body;

typedef union {
  G_Tag tag;
  union {
    G_Foo_Body foo;
    G_Bar_Body bar;
  };
} G;

typedef struct Opaque Opaque;

void root(Opaque *o, A a, B b, C c, D d, E e, F f, G g);
