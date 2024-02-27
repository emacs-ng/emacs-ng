/* Font support for Haiku windowing

Copyright (C) 2021-2023 Free Software Foundation, Inc.

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

#include "lisp.h"

/* Convert emacs script name to OTF 4-letter script code. */
Lisp_Object
script_to_otf (Lisp_Object script)
{
    Lisp_Object otf = Frassq (script, Votf_script_alist);
    return CONSP (otf)
	? SYMBOL_NAME (XCAR ((otf)))
	: Qnil;
}

/* Convert a font registry.  */
Lisp_Object
registry_to_otf (Lisp_Object reg)
{
    Lisp_Object otf, r, rsa = Vregistry_script_alist;
    while (CONSP (rsa))
      {
        r = XCAR (XCAR (rsa));
        if (!strncmp (SSDATA (r), SSDATA (SYMBOL_NAME (reg)), SBYTES (r)))
          {
	    otf = script_to_otf (XCDR (XCAR (rsa)));
            return otf;
          }
        rsa = XCDR (rsa);
      }
    return  Qnil;
}
