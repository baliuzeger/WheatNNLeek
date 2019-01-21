(uiop/package:define-package :cl-wheatnnleek-cffi/ffi
  (:use :cl)
  (:export
   :hello-world
   :sum
   :network-create
   :network-connect
   :network-static-connect
   :network-stdp-connect
   :network-record-spikes
   :network-get-spike-records
   :network-get-conn-info-by-id
   :network-run
   :network-get-population-by-id
   :network-set-static-poisson-freq
   ))
(in-package :cl-wheatnnleek-cffi/ffi)
;;;don't edit above
(cffi:define-foreign-library libwheatnnleek
  (:darwin #.(ignore-errors (namestring (probe-file (merge-pathnames "../core/target/debug/libwheatnnleek.dylib" (asdf:system-source-directory (asdf:find-system :cl-wheatnnleek-cffi nil)))))))
  (:unix #.(ignore-errors (namestring (probe-file (merge-pathnames "../core/target/debug/libwheatnnleek.so" (asdf:system-source-directory (asdf:find-system :cl-wheatnnleek-cffi nil))))))))

(cffi:use-foreign-library libwheatnnleek)
(cffi:defcfun ("hello_world" %hello_world) :pointer)
(cffi:defcfun ("json_string_free" %json_string_free) :void
  (p :pointer))

(defun hello-world ()
  (let ((p (%hello_world)))
    (unwind-protect
         (cffi:foreign-string-to-lisp p)
      (%json_string_free p))))

(cffi:defcfun ("sum" sum) :int
  (a :int)
  (b :int))

(cffi:defcfun ("Network_create" %Network_create) :pointer
  (neuron_number :int)
  (neuron_type_buf :string)
  (rests :string))

(defun network-create (neuron_number neuron_type_buf params-plist)
  (let ((p (%Network_create neuron_number neuron_type_buf (jonathan:to-json params-plist))))
    (unwind-protect
         (let ((string (cffi:foreign-string-to-lisp p)))
           (and string
                (list :|population| (jonathan:parse string))))
      (%json_string_free p))))

(cffi:defcfun ("Network_clear" network-clear) :void)

(cffi:defcfun ("Network_connect" %network-connect) :pointer
  (neuron_id1 :int)
  (neuron_id2 :int))

(defun network-connect (neuron-id1 neuron-id2)
  (let ((p (%network-connect neuron-id1 neuron-id2)))
    (unwind-protect
         (let ((string (cffi:foreign-string-to-lisp p)))
           (and string
                (jonathan:parse string)))
      (%json_string_free p))))

(cffi:defcfun ("Network_static_connect" %network-static-connect) :pointer
  (neuron_id1 :int)
  (neuron_id2 :int)
  (connection_delay :double)
  (connector :string)
  (post_syn_effect :string))

(defun network-static-connect (neuron-id1 neuron-id2 connection-delay connector post-syn-effect)
  (let ((p (%network-static-connect
            neuron-id1
            neuron-id2
            connection-delay
            connector
            post-syn-effect)))
    (unwind-protect
         (let ((string (cffi:foreign-string-to-lisp p)))
           (and string
                (jonathan:parse string)))
      (%json_string_free p))))

(cffi:defcfun ("Network_stdp_connect" %network-stdp-connect) :pointer
  (neuron_id1 :int)
  (neuron_id2 :int)
  (connection_delay :double))

(defun network-stdp-connect (neuron-id1 neuron-id2 connection-delay)
  (let ((p (%network-stdp-connect
            neuron-id1
            neuron-id2
            connection-delay)))
    (unwind-protect
         (let ((string (cffi:foreign-string-to-lisp p)))
           (and string
                (jonathan:parse string)))
      (%json_string_free p))))

(cffi:defcfun ("Network_record_spikes" network-record-spikes) :bool
  (population_id :int))

(cffi:defcfun ("Network_get_spike_records" %network-get-spike-records) :pointer)

(defun network-get-spike-records ()
  (let ((p (%network-get-spike-records)))
    (unwind-protect
         (let ((string (cffi:foreign-string-to-lisp p)))
           (and string
                (jonathan:parse string)))
      (%json_string_free p))))

(cffi:defcfun ("Network_get_conn_info_by_id" %network-get-conn-info-by-id) :pointer
  (conn-id :int))

(defun network-get-conn-info-by-id (conn-id)
  (let ((p (%network-get-conn-info-by-id conn-id)))
    (unwind-protect
         (let ((string (cffi:foreign-string-to-lisp p)))
           (and string
                (jonathan:parse string)))
      (%json_string_free p))))

(cffi:defcfun ("Network_run" network-run) :boolean
  (time :double))

(cffi:defcfun ("Network_get_population_by_id" %network-get-population-by-id) :pointer
  (population_id :int))

(defun network-get-population-by-id (population-id)
  (let ((p (%network-get-population-by-id population-id)))
    (unwind-protect
         (let ((string (cffi:foreign-string-to-lisp p)))
           (and string
                (list :|population| (jonathan:parse string))))
      (%json_string_free p))))

(cffi:defcfun ("Network_set_static_poisson_freq" network-set-static-poisson-freq) :boolean
  (neuron_id :int)
  (freq :double))
