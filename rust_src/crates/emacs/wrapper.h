#include "config.h"
/* #include <limits.h> */

#include "lisp.h"
#include "composite.h"
#include "dispextern.h"
#include "frame.h"
#include "termhooks.h"
#include "syssignal.h"
#include "coding.h"
#include "keyboard.h"
#include "puresize.h"
#include "blockinput.h"
#ifdef USE_WEBRENDER
# include "wrterm.h"
#endif
