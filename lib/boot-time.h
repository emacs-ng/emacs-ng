/* Determine the time when the machine last booted.
   Copyright (C) 2023-2024 Free Software Foundation, Inc.

   This file is free software: you can redistribute it and/or modify
   it under the terms of the GNU General Public License as published
   by the Free Software Foundation, either version 3 of the License,
   or (at your option) any later version.

   This file is distributed in the hope that it will be useful,
   but WITHOUT ANY WARRANTY; without even the implied warranty of
   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
   GNU General Public License for more details.

   You should have received a copy of the GNU General Public License
   along with this program.  If not, see <https://www.gnu.org/licenses/>.  */

/* Written by Bruno Haible <bruno@clisp.org>.  */

#ifndef _BOOT_TIME_H
#define _BOOT_TIME_H

#include <time.h>

#ifdef __cplusplus
extern "C" {
#endif


/* Store the approximate time when the machine last booted in *P_BOOT_TIME,
   and return 0.  If it cannot be determined, return -1.

   This function is not multithread-safe, since on many platforms it
   invokes the functions setutxent, getutxent, endutxent.  These
   functions are needed because they may lock FILE (so that we don't
   read garbage when a concurrent process writes to FILE), but their
   drawback is that they have a common global state.  */
extern int get_boot_time (struct timespec *p_boot_time);


#ifdef __cplusplus
}
#endif

#endif /* _BOOT_TIME_H */
