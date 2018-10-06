;; Copyright (C) 1995, 1997-1998, 2001-2017 Free Software Foundation,
(defun archive-l-e (str &optional len float)
in which case a second argument, length LEN, should be supplied.
FLOAT, if non-nil, means generate and return a float instead of an integer
\(use this for numbers that can overflow the Emacs integer)."
            result (+ (if float (* result 256.0) (ash result 8))
	     (setq result (+ (lsh result 3) (aref newmode i) (- ?0))
      (if (default-value 'enable-multibyte-characters)
	  (set-buffer-multibyte 'to))
  (let ((buffer-file-truename nil) ; avoid changing dir mtime by lock_file
(define-obsolete-function-alias 'archive-mouse-extract 'archive-extract "22.1")

		      (coding-system-for-read 'no-conversion))
	     ;; Convert to float to avoid overflow for very large files.
             (csize   (archive-l-e (+ p 15) 4 'float))
             (ucsize  (archive-l-e (+ p 25) 4 'float))
             (text    (format "  %8.0f  %-11s  %-8s  %s"
	      ;; p needs to stay an integer, since we use it in char-after
	      ;; above.  Passing through `round' limits the compressed size
	      ;; to most-positive-fixnum, but if the compressed size exceeds
	      ;; that, we cannot visit the archive anyway.
              p (+ p 29 (round csize)))))
	      (format "  %8.0f                         %d file%s"
	     ;; Convert to float to avoid overflow for very large files.
	     (csize   (archive-l-e (+ p 7) 4 'float)) ;size of a compressed file to follow (level 0 and 2),
             (ucsize  (archive-l-e (+ p 11) 4 'float))	;size of an uncompressed file.
			  (format "  %8.0f  %5S  %5S  %s"
			(format "  %10s  %8.0f  %-11s  %-8s  %s"
	       ;; p needs to stay an integer, since we use it in goto-char
	       ;; above.  Passing through `round' limits the compressed size
	       ;; to most-positive-fixnum, but if the compressed size exceeds
	       ;; that, we cannot visit the archive anyway.
	       (setq p (+ p hsize 2 (round csize))))
	       (setq p (+ p thsize 2 (round csize)))))
		(insert-unibyte (logand newval 255) (lsh newval -8))
      ;; Pay attention: the offset of Zip64 end-of-central-directory
      ;; is a 64-bit field, so it could overflow the Emacs integer
      ;; even on a 64-bit host, let alone 32-bit one.  But since we've
      ;; already read the zip file into a buffer, and this is a byte
      ;; offset into the file we've read, it must be short enough, so
      ;; such an overflow can never happen, and we can safely read
      ;; these 8 bytes into an Emacs integer.  Moreover, on host with
      ;; 32-bit Emacs integer we can only read 4 bytes, since they are
      ;; stored in little-endian byte order.
      (setq emacs-int-has-32bits (<= most-positive-fixnum #x1fffffff))
                 (archive-l-e (+ (point) 8) (if emacs-int-has-32bits 4 8))))
      (setq p (archive-l-e (+ p 48) (if emacs-int-has-32bits 4 8))))
	     ;; Convert to float to avoid overflow for very large files.
             (ucsize  (archive-l-e (+ p 24) 4 'float))
             (text    (format "  %10s  %8.0f  %-11s  %-8s  %s"
	      (format "              %8.0f                         %d file%s"
		 (insert-unibyte (logand newval 255) (lsh newval -8)))
					 (logand (logxor 1 (lsh newval -7)) 1)))
	     ;; Convert to float to avoid overflow for very large files.
             (ucsize  (archive-l-e (+ p 20) 4 'float))
             (text    (format "  %8.0f  %-11s  %-8s  %s"
	      (format "  %8.0f                         %d file%s"
      (while (looking-at (concat "^\s+[0-9.]+\s+-+\s+"   ; Flags
                                 "\\([0-9-]+\\)\s+"      ; Size
                                 "\\([0-9.%]+\\)\s+"     ; Ratio
                                 "\\([0-9a-zA-Z]+\\)\s+" ; Mode
                                 "\\([0-9-]+\\)\s+"      ; Date
                                 "\\([0-9:]+\\)\s+"      ; Time
                                 "\\(.*\\)\n"            ; Name
            ;; Emacs will automatically use float here because those
            ;; timestamps don't fit in our ints.