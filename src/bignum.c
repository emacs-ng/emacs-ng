/* Big numbers for Emacs.

Copyright 2018 Free Software Foundation, Inc.

This file is part of GNU Emacs.

GNU Emacs is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or (at
your option) any later version.

GNU Emacs is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with GNU Emacs.  If not, see <https://www.gnu.org/licenses/>.  */

#include <config.h>

#include "bignum.h"

#include "lisp.h"

#include <stdlib.h>

/* mpz global temporaries.  Making them global saves the trouble of
   properly using mpz_init and mpz_clear on temporaries even when
   storage is exhausted.  Admittedly this is not ideal.  An mpz value
   in a temporary is made permanent by mpz_swapping it with a bignum's
   value.  Although typically at most two temporaries are needed,
   rounding_driver and rounddiv_q need four altogther.  */

mpz_t mpz[4];

static void *
xrealloc_for_gmp (void *ptr, size_t ignore, size_t size)
{
  return xrealloc (ptr, size);
}

static void
xfree_for_gmp (void *ptr, size_t ignore)
{
  xfree (ptr);
}

void
init_bignum (void)
{
  eassert (mp_bits_per_limb == GMP_NUMB_BITS);
  integer_width = 1 << 16;
  mp_set_memory_functions (xmalloc, xrealloc_for_gmp, xfree_for_gmp);

  for (int i = 0; i < ARRAYELTS (mpz); i++)
    mpz_init (mpz[i]);
}

/* Return the value of the Lisp bignum N, as a double.  */
double
bignum_to_double (Lisp_Object n)
{
  return mpz_get_d (XBIGNUM (n)->value);
}

/* Return D, converted to a Lisp integer.  Discard any fraction.  */
Lisp_Object
double_to_integer (double d)
{
  mpz_set_d (mpz[0], d);
  return make_integer_mpz ();
}

/* Return a Lisp integer equal to mpz[0], which has BITS bits and which
   must not be in fixnum range.  Set mpz[0] to a junk value.  */
static Lisp_Object
make_bignum_bits (size_t bits)
{
  /* The documentation says integer-width should be nonnegative, so
     a single comparison suffices even though 'bits' is unsigned.  */
  if (integer_width < bits)
    range_error ();

  struct Lisp_Bignum *b = ALLOCATE_PSEUDOVECTOR (struct Lisp_Bignum, value,
						 PVEC_BIGNUM);
  mpz_init (b->value);
  mpz_swap (b->value, mpz[0]);
  return make_lisp_ptr (b, Lisp_Vectorlike);
}

/* Return a Lisp integer equal to mpz[0], which must not be in fixnum range.
   Set mpz[0] to a junk value.  */
static Lisp_Object
make_bignum (void)
{
  return make_bignum_bits (mpz_sizeinbase (mpz[0], 2));
}

static void mpz_set_uintmax_slow (mpz_t, uintmax_t);

/* Set RESULT to V.  */
static void
mpz_set_uintmax (mpz_t result, uintmax_t v)
{
  if (v <= ULONG_MAX)
    mpz_set_ui (result, v);
  else
    mpz_set_uintmax_slow (result, v);
}

/* Return a Lisp integer equal to N, which must not be in fixnum range.  */
Lisp_Object
make_bigint (intmax_t n)
{
  eassert (FIXNUM_OVERFLOW_P (n));
  mpz_set_intmax (mpz[0], n);
  return make_bignum ();
}
Lisp_Object
make_biguint (uintmax_t n)
{
  eassert (FIXNUM_OVERFLOW_P (n));
  mpz_set_uintmax (mpz[0], n);
  return make_bignum ();
}

/* Return a Lisp integer with value taken from mpz[0].
   Set mpz[0] to a junk value.  */
Lisp_Object
make_integer_mpz (void)
{
  size_t bits = mpz_sizeinbase (mpz[0], 2);

  if (bits <= FIXNUM_BITS)
    {
      EMACS_INT v = 0;
      int i = 0, shift = 0;

      do
	{
	  EMACS_INT limb = mpz_getlimbn (mpz[0], i++);
	  v += limb << shift;
	  shift += GMP_NUMB_BITS;
	}
      while (shift < bits);

      if (mpz_sgn (mpz[0]) < 0)
	v = -v;

      if (!FIXNUM_OVERFLOW_P (v))
	return make_fixnum (v);
    }

  return make_bignum_bits (bits);
}

/* Set RESULT to V.  This code is for when intmax_t is wider than long.  */
void
mpz_set_intmax_slow (mpz_t result, intmax_t v)
{
  int maxlimbs = (INTMAX_WIDTH + GMP_NUMB_BITS - 1) / GMP_NUMB_BITS;
  mp_limb_t *limb = mpz_limbs_write (result, maxlimbs);
  int n = 0;
  uintmax_t u = v;
  bool negative = v < 0;
  if (negative)
    {
      uintmax_t two = 2;
      u = -u & ((two << (UINTMAX_WIDTH - 1)) - 1);
    }

  do
    {
      limb[n++] = u;
      u = GMP_NUMB_BITS < UINTMAX_WIDTH ? u >> GMP_NUMB_BITS : 0;
    }
  while (u != 0);

  mpz_limbs_finish (result, negative ? -n : n);
}
static void
mpz_set_uintmax_slow (mpz_t result, uintmax_t v)
{
  int maxlimbs = (UINTMAX_WIDTH + GMP_NUMB_BITS - 1) / GMP_NUMB_BITS;
  mp_limb_t *limb = mpz_limbs_write (result, maxlimbs);
  int n = 0;

  do
    {
      limb[n++] = v;
      v = GMP_NUMB_BITS < INTMAX_WIDTH ? v >> GMP_NUMB_BITS : 0;
    }
  while (v != 0);

  mpz_limbs_finish (result, n);
}

/* Return the value of the bignum X if it fits, 0 otherwise.
   A bignum cannot be zero, so 0 indicates failure reliably.  */
intmax_t
bignum_to_intmax (Lisp_Object x)
{
  ptrdiff_t bits = mpz_sizeinbase (XBIGNUM (x)->value, 2);
  bool negative = mpz_sgn (XBIGNUM (x)->value) < 0;

  if (bits < INTMAX_WIDTH)
    {
      intmax_t v = 0;
      int i = 0, shift = 0;

      do
	{
	  intmax_t limb = mpz_getlimbn (XBIGNUM (x)->value, i++);
	  v += limb << shift;
	  shift += GMP_NUMB_BITS;
	}
      while (shift < bits);

      return negative ? -v : v;
    }
  return ((bits == INTMAX_WIDTH && INTMAX_MIN < -INTMAX_MAX && negative
	   && mpz_scan1 (XBIGNUM (x)->value, 0) == INTMAX_WIDTH - 1)
	  ? INTMAX_MIN : 0);
}
uintmax_t
bignum_to_uintmax (Lisp_Object x)
{
  uintmax_t v = 0;
  if (0 <= mpz_sgn (XBIGNUM (x)->value))
    {
      ptrdiff_t bits = mpz_sizeinbase (XBIGNUM (x)->value, 2);
      if (bits <= UINTMAX_WIDTH)
	{
	  int i = 0, shift = 0;

	  do
	    {
	      uintmax_t limb = mpz_getlimbn (XBIGNUM (x)->value, i++);
	      v += limb << shift;
	      shift += GMP_NUMB_BITS;
	    }
	  while (shift < bits);
	}
    }
  return v;
}

/* Yield an upper bound on the buffer size needed to contain a C
   string representing the bignum NUM in base BASE.  This includes any
   preceding '-' and the terminating null.  */
ptrdiff_t
bignum_bufsize (Lisp_Object num, int base)
{
  return mpz_sizeinbase (XBIGNUM (num)->value, base) + 2;
}

/* Store into BUF (of size SIZE) the value of NUM as a base-BASE string.
   If BASE is negative, use upper-case digits in base -BASE.
   Return the string's length.
   SIZE must equal bignum_bufsize (NUM, abs (BASE)).  */
ptrdiff_t
bignum_to_c_string (char *buf, ptrdiff_t size, Lisp_Object num, int base)
{
  eassert (bignum_bufsize (num, abs (base)) == size);
  mpz_get_str (buf, base, XBIGNUM (num)->value);
  ptrdiff_t n = size - 2;
  return !buf[n - 1] ? n - 1 : n + !!buf[n];
}

/* Convert NUM to a base-BASE Lisp string.
   If BASE is negative, use upper-case digits in base -BASE.  */

Lisp_Object
bignum_to_string (Lisp_Object num, int base)
{
  ptrdiff_t size = bignum_bufsize (num, abs (base));
  USE_SAFE_ALLOCA;
  char *str = SAFE_ALLOCA (size);
  ptrdiff_t len = bignum_to_c_string (str, size, num, base);
  Lisp_Object result = make_unibyte_string (str, len);
  SAFE_FREE ();
  return result;
}

/* Create a bignum by scanning NUM, with digits in BASE.
   NUM must consist of an optional '-', a nonempty sequence
   of base-BASE digits, and a terminating null byte, and
   the represented number must not be in fixnum range.  */

Lisp_Object
make_bignum_str (char const *num, int base)
{
  struct Lisp_Bignum *b = ALLOCATE_PSEUDOVECTOR (struct Lisp_Bignum, value,
						 PVEC_BIGNUM);
  mpz_init (b->value);
  int check = mpz_set_str (b->value, num, base);
  eassert (check == 0);
  return make_lisp_ptr (b, Lisp_Vectorlike);
}
