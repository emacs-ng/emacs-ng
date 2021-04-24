use crate::{bindings::*, definitions::*, lisp::LispObject};

#[allow(unused)]
#[repr(C)]
pub struct emacs_globals {
    pub V_comp_no_native_file_h: LispObject,
    pub Vafter_change_functions: LispObject,
    pub Vafter_delete_frame_functions: LispObject,
    pub Vafter_init_time: LispObject,
    pub Vafter_insert_file_functions: LispObject,
    pub Vafter_load_alist: LispObject,
    pub Valternate_fontname_alist: LispObject,
    pub Vauto_composition_function: LispObject,
    pub Vauto_composition_mode: LispObject,
    pub Vauto_fill_chars: LispObject,
    pub Vauto_resize_tab_bars: LispObject,
    pub Vauto_resize_tool_bars: LispObject,
    pub Vauto_save_include_big_deletions: LispObject,
    pub Vauto_save_list_file_name: LispObject,
    pub Vauto_save_timeout: LispObject,
    pub Vauto_save_visited_file_name: LispObject,
    pub Vbefore_change_functions: LispObject,
    pub Vbefore_init_time: LispObject,
    pub Vblink_cursor_alist: LispObject,
    pub Vbuffer_access_fontified_property: LispObject,
    pub Vbuffer_access_fontify_functions: LispObject,
    pub Vbuffer_list_update_hook: LispObject,
    pub Vbuild_files: LispObject,
    pub Vbyte_boolean_vars: LispObject,
    pub Vbyte_code_meter: LispObject,
    pub Vbytecomp_version_regexp: LispObject,
    pub Vchange_major_mode_hook: LispObject,
    pub Vchar_code_property_alist: LispObject,
    pub Vchar_property_alias_alist: LispObject,
    pub Vchar_script_table: LispObject,
    pub Vchar_width_table: LispObject,
    pub Vcharset_list: LispObject,
    pub Vcharset_map_path: LispObject,
    pub Vcharset_revision_table: LispObject,
    pub Vclear_message_function: LispObject,
    pub Vcode_conversion_map_vector: LispObject,
    pub Vcoding_category_list: LispObject,
    pub Vcoding_system_alist: LispObject,
    pub Vcoding_system_for_read: LispObject,
    pub Vcoding_system_for_write: LispObject,
    pub Vcoding_system_list: LispObject,
    pub Vcombine_after_change_calls: LispObject,
    pub Vcommand_debug_status: LispObject,
    pub Vcommand_error_function: LispObject,
    pub Vcommand_history: LispObject,
    pub Vcommand_line_args: LispObject,
    pub Vcomment_use_syntax_ppss: LispObject,
    pub Vcomp_abi_hash: LispObject,
    pub Vcomp_ctxt: LispObject,
    pub Vcomp_deferred_pending_h: LispObject,
    pub Vcomp_eln_load_path: LispObject,
    pub Vcomp_eln_to_el_h: LispObject,
    pub Vcomp_installed_trampolines_h: LispObject,
    pub Vcomp_native_version_dir: LispObject,
    pub Vcomp_subr_list: LispObject,
    pub Vcompletion_ignored_extensions: LispObject,
    pub Vcompletion_regexp_list: LispObject,
    pub Vcompose_chars_after_function: LispObject,
    pub Vcomposition_function_table: LispObject,
    pub Vconfigure_info_directory: LispObject,
    pub Vcurrent_iso639_language: LispObject,
    pub Vcurrent_load_list: LispObject,
    pub Vcurrent_minibuffer_command: LispObject,
    pub Vcurrent_prefix_arg: LispObject,
    pub Vdata_directory: LispObject,
    pub Vdbus_compiled_version: LispObject,
    pub Vdbus_debug: LispObject,
    pub Vdbus_message_type_error: LispObject,
    pub Vdbus_message_type_invalid: LispObject,
    pub Vdbus_message_type_method_call: LispObject,
    pub Vdbus_message_type_method_return: LispObject,
    pub Vdbus_message_type_signal: LispObject,
    pub Vdbus_registered_objects_table: LispObject,
    pub Vdbus_runtime_version: LispObject,
    pub Vdeactivate_mark: LispObject,
    pub Vdebug_ignored_errors: LispObject,
    pub Vdebug_on_error: LispObject,
    pub Vdebug_on_event: LispObject,
    pub Vdebug_on_message: LispObject,
    pub Vdebug_on_signal: LispObject,
    pub Vdebugger: LispObject,
    pub Vdefault_file_name_coding_system: LispObject,
    pub Vdefault_frame_alist: LispObject,
    pub Vdefault_frame_scroll_bars: LispObject,
    pub Vdefault_process_coding_system: LispObject,
    pub Vdefault_text_properties: LispObject,
    pub Vdeferred_action_function: LispObject,
    pub Vdeferred_action_list: LispObject,
    pub Vdelayed_warnings_list: LispObject,
    pub Vdelete_frame_functions: LispObject,
    pub Vdelete_terminal_functions: LispObject,
    pub Vdisable_point_adjustment: LispObject,
    pub Vdisplay_fill_column_indicator_character: LispObject,
    pub Vdisplay_fill_column_indicator_column: LispObject,
    pub Vdisplay_line_numbers: LispObject,
    pub Vdisplay_line_numbers_current_absolute: LispObject,
    pub Vdisplay_line_numbers_width: LispObject,
    pub Vdisplay_pixels_per_inch: LispObject,
    pub Vdoc_directory: LispObject,
    pub Vdoc_file_name: LispObject,
    pub Vdouble_click_time: LispObject,
    pub Vdump_mode: LispObject,
    pub Vdynamic_library_alist: LispObject,
    pub Vecho_keystrokes: LispObject,
    pub Vemacs_copyright: LispObject,
    pub Vemacs_version: LispObject,
    pub Vemulation_mode_map_alists: LispObject,
    pub Venable_character_translation: LispObject,
    pub Venable_disabled_menus_and_buttons: LispObject,
    pub Veval_buffer_list: LispObject,
    pub Vexec_directory: LispObject,
    pub Vexec_path: LispObject,
    pub Vexec_suffixes: LispObject,
    pub Vexecuting_kbd_macro: LispObject,
    pub Vface_default_stipple: LispObject,
    pub Vface_font_rescale_alist: LispObject,
    pub Vface_ignored_fonts: LispObject,
    pub Vface_new_frame_defaults: LispObject,
    pub Vface_remapping_alist: LispObject,
    pub Vfeatures: LispObject,
    pub Vfile_coding_system_alist: LispObject,
    pub Vfile_name_coding_system: LispObject,
    pub Vfile_name_handler_alist: LispObject,
    pub Vfind_word_boundary_function_table: LispObject,
    pub Vfirst_change_hook: LispObject,
    pub Vfloat_output_format: LispObject,
    pub Vfont_ccl_encoder_alist: LispObject,
    pub Vfont_encoding_alist: LispObject,
    pub Vfont_encoding_charset_alist: LispObject,
    pub Vfont_log: LispObject,
    pub Vfont_slant_table: LispObject,
    pub Vfont_weight_table: LispObject,
    pub Vfont_width_table: LispObject,
    pub Vfontification_functions: LispObject,
    pub Vfontset_alias_alist: LispObject,
    pub Vframe_alpha_lower_limit: LispObject,
    pub Vframe_title_format: LispObject,
    pub Vfringe_bitmaps: LispObject,
    pub Vfunction_key_map: LispObject,
    pub Vgc_cons_percentage: LispObject,
    pub Vgc_elapsed: LispObject,
    pub Vglobal_disable_point_adjustment: LispObject,
    pub Vglobal_mode_string: LispObject,
    pub Vglyph_table: LispObject,
    pub Vglyphless_char_display: LispObject,
    pub Vhelp_char: LispObject,
    pub Vhelp_event_list: LispObject,
    pub Vhelp_form: LispObject,
    pub Vhistory_add_new_input: LispObject,
    pub Vhistory_length: LispObject,
    pub Vhourglass_delay: LispObject,
    pub Vhscroll_step: LispObject,
    pub Vicon_title_format: LispObject,
    pub Vignore_relative_composition: LispObject,
    pub Vimage_cache_eviction_delay: LispObject,
    pub Vimage_types: LispObject,
    pub Vinhibit_changing_match_data: LispObject,
    pub Vinhibit_debugger: LispObject,
    pub Vinhibit_field_text_motion: LispObject,
    pub Vinhibit_file_name_handlers: LispObject,
    pub Vinhibit_file_name_operation: LispObject,
    pub Vinhibit_point_motion_hooks: LispObject,
    pub Vinhibit_quit: LispObject,
    pub Vinhibit_read_only: LispObject,
    pub Vinhibit_redisplay: LispObject,
    pub Vinitial_environment: LispObject,
    pub Vinitial_window_system: LispObject,
    pub Vinput_method_function: LispObject,
    pub Vinput_method_previous_message: LispObject,
    pub Vinstallation_directory: LispObject,
    pub Vinternal__daemon_sockname: LispObject,
    pub Vinternal__top_level_message: LispObject,
    pub Vinternal_interpreter_environment: LispObject,
    pub Vinterrupt_process_functions: LispObject,
    pub Vinvocation_directory: LispObject,
    pub Vinvocation_name: LispObject,
    pub Vjs_retain_map: LispObject,
    pub Vkbd_macro_termination_hook: LispObject,
    pub Vkey_translation_map: LispObject,
    pub Vkill_buffer_query_functions: LispObject,
    pub Vkill_emacs_hook: LispObject,
    pub Vlast_code_conversion_error: LispObject,
    pub Vlast_coding_system_used: LispObject,
    pub Vlast_event_frame: LispObject,
    pub Vlatin_extra_code_table: LispObject,
    pub Vlexical_binding: LispObject,
    pub Vlibgnutls_version: LispObject,
    pub Vline_number_display_limit: LispObject,
    pub Vline_prefix: LispObject,
    pub Vload_file_name: LispObject,
    pub Vload_file_rep_suffixes: LispObject,
    pub Vload_history: LispObject,
    pub Vload_path: LispObject,
    pub Vload_read_function: LispObject,
    pub Vload_source_file_function: LispObject,
    pub Vload_suffixes: LispObject,
    pub Vload_true_file_name: LispObject,
    pub Vlocale_coding_system: LispObject,
    pub Vlread_unescaped_character_literals: LispObject,
    pub Vlucid_menu_bar_dirty_flag: LispObject,
    pub Vmain_thread: LispObject,
    pub Vmake_cursor_line_fully_visible: LispObject,
    pub Vmake_pointer_invisible: LispObject,
    pub Vmark_even_if_inactive: LispObject,
    pub Vmax_image_size: LispObject,
    pub Vmax_mini_window_height: LispObject,
    pub Vmaximum_scroll_margin: LispObject,
    pub Vmemory_full: LispObject,
    pub Vmemory_signal_data: LispObject,
    pub Vmenu_bar_final_items: LispObject,
    pub Vmenu_bar_mode: LispObject,
    pub Vmenu_bar_update_hook: LispObject,
    pub Vmenu_updating_frame: LispObject,
    pub Vmessage_log_max: LispObject,
    pub Vminibuf_scroll_window: LispObject,
    pub Vminibuffer_completing_file_name: LispObject,
    pub Vminibuffer_completion_confirm: LispObject,
    pub Vminibuffer_completion_predicate: LispObject,
    pub Vminibuffer_completion_table: LispObject,
    pub Vminibuffer_exit_hook: LispObject,
    pub Vminibuffer_help_form: LispObject,
    pub Vminibuffer_history_position: LispObject,
    pub Vminibuffer_history_variable: LispObject,
    pub Vminibuffer_local_map: LispObject,
    pub Vminibuffer_local_ns_map: LispObject,
    pub Vminibuffer_message_timeout: LispObject,
    pub Vminibuffer_prompt_properties: LispObject,
    pub Vminibuffer_setup_hook: LispObject,
    pub Vminor_mode_map_alist: LispObject,
    pub Vminor_mode_overriding_map_alist: LispObject,
    pub Vmode_line_compact: LispObject,
    pub Vmodule_file_suffix: LispObject,
    pub Vmost_negative_fixnum: LispObject,
    pub Vmost_positive_fixnum: LispObject,
    pub Vmouse_autoselect_window: LispObject,
    pub Vmouse_highlight: LispObject,
    pub Vmouse_leave_buffer_hook: LispObject,
    pub Vmouse_position_function: LispObject,
    pub Vmove_frame_functions: LispObject,
    pub Vnetwork_coding_system_alist: LispObject,
    pub Vnobreak_char_display: LispObject,
    pub Vobarray: LispObject,
    pub Voperating_system_release: LispObject,
    pub Votf_script_alist: LispObject,
    pub Vother_window_scroll_buffer: LispObject,
    pub Voverflow_newline_into_fringe: LispObject,
    pub Voverlay_arrow_position: LispObject,
    pub Voverlay_arrow_string: LispObject,
    pub Voverlay_arrow_variable_list: LispObject,
    pub Voverriding_local_map: LispObject,
    pub Voverriding_local_map_menu_flag: LispObject,
    pub Voverriding_plist_environment: LispObject,
    pub Vpath_separator: LispObject,
    pub Vpost_command_hook: LispObject,
    pub Vpost_gc_hook: LispObject,
    pub Vpost_self_insert_hook: LispObject,
    pub Vpre_command_hook: LispObject,
    pub Vpre_redisplay_function: LispObject,
    pub Vprefix_help_command: LispObject,
    pub Vpreloaded_file_list: LispObject,
    pub Vprint_charset_text_property: LispObject,
    pub Vprint_circle: LispObject,
    pub Vprint_continuous_numbering: LispObject,
    pub Vprint_gensym: LispObject,
    pub Vprint_length: LispObject,
    pub Vprint_level: LispObject,
    pub Vprint_number_table: LispObject,
    pub Vprintable_chars: LispObject,
    pub Vprocess_adaptive_read_buffering: LispObject,
    pub Vprocess_coding_system_alist: LispObject,
    pub Vprocess_connection_type: LispObject,
    pub Vprocess_environment: LispObject,
    pub Vpurify_flag: LispObject,
    pub Vquit_flag: LispObject,
    pub Vread_buffer_function: LispObject,
    pub Vread_circle: LispObject,
    pub Vread_expression_history: LispObject,
    pub Vread_hide_char: LispObject,
    pub Vread_symbol_positions_list: LispObject,
    pub Vread_with_symbol_positions: LispObject,
    pub Vreal_this_command: LispObject,
    pub Vrecenter_redisplay: LispObject,
    pub Vredisplay__all_windows_cause: LispObject,
    pub Vredisplay__mode_lines_cause: LispObject,
    pub Vredisplay_end_trigger_functions: LispObject,
    pub Vregion_extract_function: LispObject,
    pub Vreport_emacs_bug_address: LispObject,
    pub Vresize_mini_windows: LispObject,
    pub Vresume_tty_functions: LispObject,
    pub Vring_bell_function: LispObject,
    pub Vsaved_region_selection: LispObject,
    pub Vscalable_fonts_allowed: LispObject,
    pub Vscript_representative_chars: LispObject,
    pub Vscroll_preserve_screen_position: LispObject,
    pub Vsearch_spaces_regexp: LispObject,
    pub Vselect_active_regions: LispObject,
    pub Vselect_safe_coding_system_function: LispObject,
    pub Vselection_inhibit_update_commands: LispObject,
    pub Vset_auto_coding_function: LispObject,
    pub Vset_message_function: LispObject,
    pub Vshared_game_score_directory: LispObject,
    pub Vshell_file_name: LispObject,
    pub Vshow_help_function: LispObject,
    pub Vshow_trailing_whitespace: LispObject,
    pub Vsignal_hook_function: LispObject,
    pub Vsource_directory: LispObject,
    pub Vspecial_event_map: LispObject,
    pub Vstandard_display_table: LispObject,
    pub Vstandard_input: LispObject,
    pub Vstandard_output: LispObject,
    pub Vstandard_translation_table_for_decode: LispObject,
    pub Vstandard_translation_table_for_encode: LispObject,
    pub Vsuspend_tty_functions: LispObject,
    pub Vsystem_configuration: LispObject,
    pub Vsystem_configuration_features: LispObject,
    pub Vsystem_configuration_options: LispObject,
    pub Vsystem_messages_locale: LispObject,
    pub Vsystem_name: LispObject,
    pub Vsystem_time_locale: LispObject,
    pub Vsystem_type: LispObject,
    pub Vtab_bar_border: LispObject,
    pub Vtab_bar_button_margin: LispObject,
    pub Vtab_bar_mode: LispObject,
    pub Vtab_bar_position: LispObject,
    pub Vtab_bar_separator_image_expression: LispObject,
    pub Vtemp_buffer_show_function: LispObject,
    pub Vtemporary_file_directory: LispObject,
    pub Vterminal_frame: LispObject,
    pub Vtext_property_default_nonsticky: LispObject,
    pub Vtext_quoting_style: LispObject,
    pub Vthis_command: LispObject,
    pub Vthis_command_keys_shift_translated: LispObject,
    pub Vthis_original_command: LispObject,
    pub Vthrow_on_input: LispObject,
    pub Vtimer_idle_list: LispObject,
    pub Vtimer_list: LispObject,
    pub Vtool_bar_border: LispObject,
    pub Vtool_bar_button_margin: LispObject,
    pub Vtool_bar_mode: LispObject,
    pub Vtool_bar_separator_image_expression: LispObject,
    pub Vtool_bar_style: LispObject,
    pub Vtop_level: LispObject,
    pub Vtransient_mark_mode: LispObject,
    pub Vtranslation_hash_table_vector: LispObject,
    pub Vtranslation_table_for_input: LispObject,
    pub Vtranslation_table_vector: LispObject,
    pub Vtruncate_partial_width_windows: LispObject,
    pub Vtty_defined_color_alist: LispObject,
    pub Vtty_erase_char: LispObject,
    pub Vundo_outer_limit: LispObject,
    pub Vundo_outer_limit_function: LispObject,
    pub Vunicode_category_table: LispObject,
    pub Vunread_command_events: LispObject,
    pub Vunread_input_method_events: LispObject,
    pub Vunread_post_input_method_events: LispObject,
    pub Vuse_default_ascent: LispObject,
    pub Vuser_full_name: LispObject,
    pub Vuser_init_file: LispObject,
    pub Vuser_login_name: LispObject,
    pub Vuser_real_login_name: LispObject,
    pub Vvalues: LispObject,
    pub Vvertical_centering_font_regexp: LispObject,
    pub Vvoid_text_area_pointer: LispObject,
    pub Vwhere_is_preferred_modifier: LispObject,
    pub Vwhile_no_input_ignore_events: LispObject,
    pub Vwindow_buffer_change_functions: LispObject,
    pub Vwindow_combination_limit: LispObject,
    pub Vwindow_combination_resize: LispObject,
    pub Vwindow_configuration_change_hook: LispObject,
    pub Vwindow_persistent_parameters: LispObject,
    pub Vwindow_point_insertion_type: LispObject,
    pub Vwindow_scroll_functions: LispObject,
    pub Vwindow_selection_change_functions: LispObject,
    pub Vwindow_size_change_functions: LispObject,
    pub Vwindow_state_change_functions: LispObject,
    pub Vwindow_state_change_hook: LispObject,
    pub Vwindow_system_version: LispObject,
    pub Vword_combining_categories: LispObject,
    pub Vword_separating_categories: LispObject,
    pub Vwrap_prefix: LispObject,
    pub Vwrite_region_annotate_functions: LispObject,
    pub Vwrite_region_annotations_so_far: LispObject,
    pub Vwrite_region_post_annotation_function: LispObject,
    pub Vx_bitmap_file_path: LispObject,
    pub Vx_keysym_table: LispObject,
    pub Vx_resource_class: LispObject,
    pub Vx_resource_name: LispObject,
    pub Vx_toolkit_scroll_bars: LispObject,
    pub automatic_hscrolling: LispObject,
    pub eol_mnemonic_dos: LispObject,
    pub eol_mnemonic_mac: LispObject,
    pub eol_mnemonic_undecided: LispObject,
    pub eol_mnemonic_unix: LispObject,
    pub focus_follows_mouse: LispObject,
    pub frame_inhibit_implied_resize: LispObject,
    pub frame_size_history: LispObject,
    pub iconify_child_frame: LispObject,
    pub last_command_event: LispObject,
    pub last_input_event: LispObject,
    pub last_nonmenu_event: LispObject,
    pub menu_prompt_more_char: LispObject,
    pub meta_prefix_char: LispObject,
    pub minibuffer_follows_selected_frame: LispObject,
    pub resize_mini_frames: LispObject,
    pub track_mouse: LispObject,
    pub auto_save_interval: intmax_t,
    pub baud_rate: intmax_t,
    pub cons_cells_consed: intmax_t,
    pub debug_end_pos: intmax_t,
    pub display_line_numbers_major_tick: intmax_t,
    pub display_line_numbers_minor_tick: intmax_t,
    pub display_line_numbers_offset: intmax_t,
    pub double_click_fuzz: intmax_t,
    pub emacs_scroll_step: intmax_t,
    pub executing_kbd_macro_index: intmax_t,
    pub extra_keyboard_modifiers: intmax_t,
    pub face_near_same_color_threshold: intmax_t,
    pub floats_consed: intmax_t,
    pub gc_cons_threshold: intmax_t,
    pub gcs_done: intmax_t,
    pub global_gnutls_log_level: intmax_t,
    pub hscroll_margin: intmax_t,
    pub imagemagick_render_type: intmax_t,
    pub integer_width: intmax_t,
    pub intervals_consed: intmax_t,
    pub line_number_display_limit_width: intmax_t,
    pub max_lisp_eval_depth: intmax_t,
    pub max_specpdl_size: intmax_t,
    pub next_screen_context_lines: intmax_t,
    pub num_input_keys: intmax_t,
    pub num_nonmacro_input_events: intmax_t,
    pub overline_margin: intmax_t,
    pub polling_period: intmax_t,
    pub profiler_log_size: intmax_t,
    pub profiler_max_stack_depth: intmax_t,
    pub pure_bytes_used: intmax_t,
    pub read_process_output_max: intmax_t,
    pub scroll_conservatively: intmax_t,
    pub scroll_margin: intmax_t,
    pub string_chars_consed: intmax_t,
    pub strings_consed: intmax_t,
    pub symbols_consed: intmax_t,
    pub syntax_propertize__done: intmax_t,
    pub tab_bar_button_relief: intmax_t,
    pub tool_bar_button_relief: intmax_t,
    pub tool_bar_max_label_size: intmax_t,
    pub underline_minimum_offset: intmax_t,
    pub undo_limit: intmax_t,
    pub undo_strong_limit: intmax_t,
    pub vector_cells_consed: intmax_t,
    pub when_entered_debugger: intmax_t,
    pub Vx_underline_at_descent_line: bool,
    pub Vx_use_underline_position_properties: bool,
    pub attempt_orderly_shutdown_on_fatal_signal: bool,
    pub attempt_stack_overflow_recovery: bool,
    pub auto_raise_tab_bar_buttons_p: bool,
    pub auto_raise_tool_bar_buttons_p: bool,
    pub auto_save_no_message: bool,
    pub auto_window_vscroll_p: bool,
    pub backtrace_on_error_noninteractive: bool,
    pub bidi_inhibit_bpa: bool,
    pub binary_as_unsigned: bool,
    pub byte_metering_on: bool,
    pub cannot_suspend: bool,
    pub coding_system_require_warning: bool,
    pub comment_end_can_be_escaped: bool,
    pub comp_deferred_compilation: bool,
    pub comp_enable_subr_trampolines: bool,
    pub completion_ignore_case: bool,
    pub create_lockfiles: bool,
    pub cross_disabled_images: bool,
    pub cursor_in_echo_area: bool,
    pub debug_on_next_call: bool,
    pub debug_on_quit: bool,
    pub debugger_may_continue: bool,
    pub debugger_stack_frame_as_list: bool,
    pub delete_by_moving_to_trash: bool,
    pub delete_exited_processes: bool,
    pub disable_ascii_optimization: bool,
    pub display_fill_column_indicator: bool,
    pub display_hourglass_p: bool,
    pub display_line_numbers_widen: bool,
    pub display_raw_bytes_as_hex: bool,
    pub enable_recursive_minibuffers: bool,
    pub face_filters_always_match: bool,
    pub fast_but_imprecise_scrolling: bool,
    pub force_load_messages: bool,
    pub frame_resize_pixelwise: bool,
    pub garbage_collection_messages: bool,
    pub highlight_nonselected_windows: bool,
    pub history_delete_duplicates: bool,
    pub indent_tabs_mode: bool,
    pub inherit_process_coding_system: bool,
    pub inhibit_bidi_mirroring: bool,
    pub inhibit_compacting_font_caches: bool,
    pub inhibit_eol_conversion: bool,
    pub inhibit_eval_during_redisplay: bool,
    pub inhibit_free_realized_faces: bool,
    pub inhibit_interaction: bool,
    pub inhibit_iso_escape_detection: bool,
    pub inhibit_load_charset_map: bool,
    pub inhibit_menubar_update: bool,
    pub inhibit_message: bool,
    pub inhibit_modification_hooks: bool,
    pub inhibit_null_byte_detection: bool,
    pub inhibit_record_char: bool,
    pub inhibit_try_cursor_movement: bool,
    pub inhibit_try_window_id: bool,
    pub inhibit_try_window_reusing: bool,
    pub inhibit_x_resources: bool,
    pub inverse_video: bool,
    pub load_convert_to_unibyte: bool,
    pub load_dangerous_libraries: bool,
    pub load_force_doc_strings: bool,
    pub load_in_progress: bool,
    pub load_no_native: bool,
    pub load_prefer_newer: bool,
    pub menu_prompting: bool,
    pub message_truncate_lines: bool,
    pub minibuffer_allow_text_properties: bool,
    pub minibuffer_auto_raise: bool,
    pub mode_line_in_non_selected_windows: bool,
    pub mouse_fine_grained_tracking: bool,
    pub multibyte_syntax_as_symbol: bool,
    pub multiple_frames: bool,
    pub no_redraw_on_reenter: bool,
    pub noninteractive1: bool,
    pub open_paren_in_column_0_is_defun_start: bool,
    pub parse_sexp_ignore_comments: bool,
    pub parse_sexp_lookup_properties: bool,
    pub print_escape_control_characters: bool,
    pub print_escape_multibyte: bool,
    pub print_escape_newlines: bool,
    pub print_escape_nonascii: bool,
    pub print_integers_as_characters: bool,
    pub print_quoted: bool,
    pub query_all_font_backends: bool,
    pub read_buffer_completion_ignore_case: bool,
    pub redisplay__inhibit_bidi: bool,
    pub redisplay_adhoc_scroll_in_resize_mini_windows: bool,
    pub redisplay_dont_pause: bool,
    pub redisplay_skip_fontification_on_input: bool,
    pub redisplay_skip_initial_frame: bool,
    pub scroll_bar_adjust_thumb_portion_p: bool,
    pub scroll_minibuffer_conservatively: bool,
    pub system_uses_terminfo: bool,
    pub text_quoting_flag: bool,
    pub tooltip_reuse_hidden_frame: bool,
    pub tty_menu_calls_mouse_position_function: bool,
    pub undo_inhibit_record_point: bool,
    pub unibyte_display_via_language_environment: bool,
    pub use_default_font_for_symbols: bool,
    pub use_dialog_box: bool,
    pub use_file_dialog: bool,
    pub use_short_answers: bool,
    pub visible_bell: bool,
    pub visible_cursor: bool,
    pub window_resize_pixelwise: bool,
    pub word_wrap_by_category: bool,
    pub words_include_escapes: bool,
    pub write_region_inhibit_fsync: bool,
    pub x_stretch_cursor_p: bool,
    pub xft_ignore_color_fonts: bool,
}
pub const Qnil: LispObject =
    crate::lisp::LispObject(0 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qt: LispObject =
    crate::lisp::LispObject(1 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qunbound: LispObject =
    crate::lisp::LispObject(2 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qerror: LispObject =
    crate::lisp::LispObject(3 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qlambda: LispObject =
    crate::lisp::LispObject(4 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QAutomatic_GC: LispObject =
    crate::lisp::LispObject(5 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCadstyle: LispObject =
    crate::lisp::LispObject(6 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCadvertised_binding: LispObject =
    crate::lisp::LispObject(7 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCalign_to: LispObject =
    crate::lisp::LispObject(8 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCallow_net: LispObject =
    crate::lisp::LispObject(9 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCallow_read: LispObject =
    crate::lisp::LispObject(10 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCallow_run: LispObject =
    crate::lisp::LispObject(11 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCallow_write: LispObject =
    crate::lisp::LispObject(12 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCantialias: LispObject =
    crate::lisp::LispObject(13 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCarray: LispObject =
    crate::lisp::LispObject(14 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCarray_type: LispObject =
    crate::lisp::LispObject(15 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCascent: LispObject =
    crate::lisp::LispObject(16 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCascii_compatible_p: LispObject =
    crate::lisp::LispObject(17 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCavgwidth: LispObject =
    crate::lisp::LispObject(18 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCbackground: LispObject =
    crate::lisp::LispObject(19 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCbase_uri: LispObject =
    crate::lisp::LispObject(20 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCbold: LispObject =
    crate::lisp::LispObject(21 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCboolean: LispObject =
    crate::lisp::LispObject(22 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCbox: LispObject =
    crate::lisp::LispObject(23 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCbuffer: LispObject =
    crate::lisp::LispObject(24 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCbutton: LispObject =
    crate::lisp::LispObject(25 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCbyte: LispObject =
    crate::lisp::LispObject(26 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCbytesize: LispObject =
    crate::lisp::LispObject(27 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCcategory: LispObject =
    crate::lisp::LispObject(28 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCcipher_aead_capable: LispObject =
    crate::lisp::LispObject(29 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCcipher_blocksize: LispObject =
    crate::lisp::LispObject(30 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCcipher_id: LispObject =
    crate::lisp::LispObject(31 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCcipher_ivsize: LispObject =
    crate::lisp::LispObject(32 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCcipher_keysize: LispObject =
    crate::lisp::LispObject(33 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCcipher_tagsize: LispObject =
    crate::lisp::LispObject(34 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCcoding: LispObject =
    crate::lisp::LispObject(35 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCcolor: LispObject =
    crate::lisp::LispObject(36 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCcolor_adjustment: LispObject =
    crate::lisp::LispObject(37 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCcolor_symbols: LispObject =
    crate::lisp::LispObject(38 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCcombining_capability: LispObject =
    crate::lisp::LispObject(39 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCcommand: LispObject =
    crate::lisp::LispObject(40 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCcomplete_negotiation: LispObject =
    crate::lisp::LispObject(41 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCconnection_type: LispObject =
    crate::lisp::LispObject(42 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCconversion: LispObject =
    crate::lisp::LispObject(43 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCcrlfiles: LispObject =
    crate::lisp::LispObject(44 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCcrop: LispObject =
    crate::lisp::LispObject(45 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCdata: LispObject =
    crate::lisp::LispObject(46 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCdebug_on_exit: LispObject =
    crate::lisp::LispObject(47 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCdecode_translation_table: LispObject =
    crate::lisp::LispObject(48 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCdefault_char: LispObject =
    crate::lisp::LispObject(49 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCdevice: LispObject =
    crate::lisp::LispObject(50 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCdict_entry: LispObject =
    crate::lisp::LispObject(51 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCdigest_algorithm_id: LispObject =
    crate::lisp::LispObject(52 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCdigest_algorithm_length: LispObject =
    crate::lisp::LispObject(53 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCdistant_foreground: LispObject =
    crate::lisp::LispObject(54 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCdocumentation: LispObject =
    crate::lisp::LispObject(55 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCdouble: LispObject =
    crate::lisp::LispObject(56 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCdpi: LispObject =
    crate::lisp::LispObject(57 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCenable: LispObject =
    crate::lisp::LispObject(58 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCencode_translation_table: LispObject =
    crate::lisp::LispObject(59 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCeval: LispObject =
    crate::lisp::LispObject(60 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCextend: LispObject =
    crate::lisp::LispObject(61 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCfalse: LispObject =
    crate::lisp::LispObject(62 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCfalse_object: LispObject =
    crate::lisp::LispObject(63 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCfamily: LispObject =
    crate::lisp::LispObject(64 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCfile: LispObject =
    crate::lisp::LispObject(65 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCfile_handler: LispObject =
    crate::lisp::LispObject(66 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCfilter: LispObject =
    crate::lisp::LispObject(67 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCfiltered: LispObject =
    crate::lisp::LispObject(68 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCflowcontrol: LispObject =
    crate::lisp::LispObject(69 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCfont: LispObject =
    crate::lisp::LispObject(70 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCfont_entity: LispObject =
    crate::lisp::LispObject(71 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCfontset: LispObject =
    crate::lisp::LispObject(72 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCforeground: LispObject =
    crate::lisp::LispObject(73 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCfoundry: LispObject =
    crate::lisp::LispObject(74 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCheight: LispObject =
    crate::lisp::LispObject(75 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QChelp: LispObject =
    crate::lisp::LispObject(76 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCheuristic_mask: LispObject =
    crate::lisp::LispObject(77 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QChost: LispObject =
    crate::lisp::LispObject(78 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QChostname: LispObject =
    crate::lisp::LispObject(79 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCignore_defface: LispObject =
    crate::lisp::LispObject(80 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCimage: LispObject =
    crate::lisp::LispObject(81 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCinchannel: LispObject =
    crate::lisp::LispObject(82 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCindex: LispObject =
    crate::lisp::LispObject(83 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCinherit: LispObject =
    crate::lisp::LispObject(84 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCinspect: LispObject =
    crate::lisp::LispObject(85 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCinspect_brk: LispObject =
    crate::lisp::LispObject(86 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCint16: LispObject =
    crate::lisp::LispObject(87 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCint32: LispObject =
    crate::lisp::LispObject(88 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCint64: LispObject =
    crate::lisp::LispObject(89 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCinverse_video: LispObject =
    crate::lisp::LispObject(90 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCitalic: LispObject =
    crate::lisp::LispObject(91 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCjs_error_handler: LispObject =
    crate::lisp::LispObject(92 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCjs_tick_rate: LispObject =
    crate::lisp::LispObject(93 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCjson_config: LispObject =
    crate::lisp::LispObject(94 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCkey_sequence: LispObject =
    crate::lisp::LispObject(95 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCkeylist: LispObject =
    crate::lisp::LispObject(96 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCkeys: LispObject =
    crate::lisp::LispObject(97 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QClabel: LispObject =
    crate::lisp::LispObject(98 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QClang: LispObject =
    crate::lisp::LispObject(99 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCline_width: LispObject =
    crate::lisp::LispObject(100 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCloader: LispObject =
    crate::lisp::LispObject(101 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QClocal: LispObject =
    crate::lisp::LispObject(102 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QClog: LispObject =
    crate::lisp::LispObject(103 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCloglevel: LispObject =
    crate::lisp::LispObject(104 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCloops_per_tick: LispObject =
    crate::lisp::LispObject(105 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCmac_algorithm_id: LispObject =
    crate::lisp::LispObject(106 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCmac_algorithm_keysize: LispObject =
    crate::lisp::LispObject(107 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCmac_algorithm_length: LispObject =
    crate::lisp::LispObject(108 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCmac_algorithm_noncesize: LispObject =
    crate::lisp::LispObject(109 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCmap: LispObject =
    crate::lisp::LispObject(110 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCmargin: LispObject =
    crate::lisp::LispObject(111 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCmask: LispObject =
    crate::lisp::LispObject(112 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCmatrix: LispObject =
    crate::lisp::LispObject(113 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCmax_height: LispObject =
    crate::lisp::LispObject(114 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCmax_width: LispObject =
    crate::lisp::LispObject(115 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCmethod: LispObject =
    crate::lisp::LispObject(116 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCmin_prime_bits: LispObject =
    crate::lisp::LispObject(117 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCmnemonic: LispObject =
    crate::lisp::LispObject(118 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCmonitor: LispObject =
    crate::lisp::LispObject(119 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCname: LispObject =
    crate::lisp::LispObject(120 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCno_check: LispObject =
    crate::lisp::LispObject(121 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCno_remote: LispObject =
    crate::lisp::LispObject(122 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCnoquery: LispObject =
    crate::lisp::LispObject(123 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCnowait: LispObject =
    crate::lisp::LispObject(124 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCnull: LispObject =
    crate::lisp::LispObject(125 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCnull_object: LispObject =
    crate::lisp::LispObject(126 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCobject_path: LispObject =
    crate::lisp::LispObject(127 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCobject_type: LispObject =
    crate::lisp::LispObject(128 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCotf: LispObject =
    crate::lisp::LispObject(129 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCoutchannel: LispObject =
    crate::lisp::LispObject(130 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCoverline: LispObject =
    crate::lisp::LispObject(131 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCparity: LispObject =
    crate::lisp::LispObject(132 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCplist: LispObject =
    crate::lisp::LispObject(133 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCpointer: LispObject =
    crate::lisp::LispObject(134 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCport: LispObject =
    crate::lisp::LispObject(135 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCpost_read_conversion: LispObject =
    crate::lisp::LispObject(136 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCpre_write_conversion: LispObject =
    crate::lisp::LispObject(137 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCpriority: LispObject =
    crate::lisp::LispObject(138 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCprocess: LispObject =
    crate::lisp::LispObject(139 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCpropertize: LispObject =
    crate::lisp::LispObject(140 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCpt_height: LispObject =
    crate::lisp::LispObject(141 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCpt_width: LispObject =
    crate::lisp::LispObject(142 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCpurecopy: LispObject =
    crate::lisp::LispObject(143 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCradio: LispObject =
    crate::lisp::LispObject(144 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCregistry: LispObject =
    crate::lisp::LispObject(145 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCrehash_size: LispObject =
    crate::lisp::LispObject(146 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCrehash_threshold: LispObject =
    crate::lisp::LispObject(147 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCrelative_height: LispObject =
    crate::lisp::LispObject(148 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCrelative_width: LispObject =
    crate::lisp::LispObject(149 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCrelief: LispObject =
    crate::lisp::LispObject(150 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCremote: LispObject =
    crate::lisp::LispObject(151 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCreverse_video: LispObject =
    crate::lisp::LispObject(152 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCrotation: LispObject =
    crate::lisp::LispObject(153 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCrtl: LispObject =
    crate::lisp::LispObject(154 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCscalable: LispObject =
    crate::lisp::LispObject(155 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCscale: LispObject =
    crate::lisp::LispObject(156 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCscript: LispObject =
    crate::lisp::LispObject(157 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCsentinel: LispObject =
    crate::lisp::LispObject(158 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCserial: LispObject =
    crate::lisp::LispObject(159 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCserver: LispObject =
    crate::lisp::LispObject(160 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCservice: LispObject =
    crate::lisp::LispObject(161 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCsession: LispObject =
    crate::lisp::LispObject(162 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCsession_private: LispObject =
    crate::lisp::LispObject(163 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCsignal: LispObject =
    crate::lisp::LispObject(164 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCsignature: LispObject =
    crate::lisp::LispObject(165 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCsize: LispObject =
    crate::lisp::LispObject(166 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCslant: LispObject =
    crate::lisp::LispObject(167 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCspacing: LispObject =
    crate::lisp::LispObject(168 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCspeed: LispObject =
    crate::lisp::LispObject(169 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCstderr: LispObject =
    crate::lisp::LispObject(170 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCstipple: LispObject =
    crate::lisp::LispObject(171 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCstop: LispObject =
    crate::lisp::LispObject(172 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCstopbits: LispObject =
    crate::lisp::LispObject(173 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCstrike_through: LispObject =
    crate::lisp::LispObject(174 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCstring: LispObject =
    crate::lisp::LispObject(175 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCstruct: LispObject =
    crate::lisp::LispObject(176 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCstyle: LispObject =
    crate::lisp::LispObject(177 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCsummary: LispObject =
    crate::lisp::LispObject(178 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCsystem: LispObject =
    crate::lisp::LispObject(179 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCsystem_private: LispObject =
    crate::lisp::LispObject(180 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCtest: LispObject =
    crate::lisp::LispObject(181 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCtimeout: LispObject =
    crate::lisp::LispObject(182 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCtls_parameters: LispObject =
    crate::lisp::LispObject(183 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCtoggle: LispObject =
    crate::lisp::LispObject(184 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCtransform_smoothing: LispObject =
    crate::lisp::LispObject(185 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCtrustfiles: LispObject =
    crate::lisp::LispObject(186 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCts_config: LispObject =
    crate::lisp::LispObject(187 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCtype: LispObject =
    crate::lisp::LispObject(188 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCtypescript: LispObject =
    crate::lisp::LispObject(189 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCuint16: LispObject =
    crate::lisp::LispObject(190 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCuint32: LispObject =
    crate::lisp::LispObject(191 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCuint64: LispObject =
    crate::lisp::LispObject(192 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCunderline: LispObject =
    crate::lisp::LispObject(193 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCunix_fd: LispObject =
    crate::lisp::LispObject(194 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCuse_color: LispObject =
    crate::lisp::LispObject(195 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCuse_external_socket: LispObject =
    crate::lisp::LispObject(196 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCuser_spec: LispObject =
    crate::lisp::LispObject(197 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCvariant: LispObject =
    crate::lisp::LispObject(198 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCverify_error: LispObject =
    crate::lisp::LispObject(199 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCverify_flags: LispObject =
    crate::lisp::LispObject(200 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCvert_only: LispObject =
    crate::lisp::LispObject(201 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCvisible: LispObject =
    crate::lisp::LispObject(202 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCvolume: LispObject =
    crate::lisp::LispObject(203 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCweakness: LispObject =
    crate::lisp::LispObject(204 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCweight: LispObject =
    crate::lisp::LispObject(205 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCwidth: LispObject =
    crate::lisp::LispObject(206 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QCwindow: LispObject =
    crate::lisp::LispObject(207 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QEmacsFrameResize: LispObject =
    crate::lisp::LispObject(208 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QL2R: LispObject =
    crate::lisp::LispObject(209 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QPRIMARY: LispObject =
    crate::lisp::LispObject(210 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const QR2L: LispObject =
    crate::lisp::LispObject(211 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qabove: LispObject =
    crate::lisp::LispObject(212 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qabove_handle: LispObject =
    crate::lisp::LispObject(213 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qabove_suspended: LispObject =
    crate::lisp::LispObject(214 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qaccess: LispObject =
    crate::lisp::LispObject(215 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qaccess_file: LispObject =
    crate::lisp::LispObject(216 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qactivate_input_method: LispObject =
    crate::lisp::LispObject(217 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qactivate_menubar_hook: LispObject =
    crate::lisp::LispObject(218 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qadd1: LispObject =
    crate::lisp::LispObject(219 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qadd_name_to_file: LispObject =
    crate::lisp::LispObject(220 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qadjust_frame_size_1: LispObject =
    crate::lisp::LispObject(221 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qadjust_frame_size_2: LispObject =
    crate::lisp::LispObject(222 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qadjust_frame_size_3: LispObject =
    crate::lisp::LispObject(223 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qafter_change_functions: LispObject =
    crate::lisp::LispObject(224 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qafter_delete_frame_functions: LispObject =
    crate::lisp::LispObject(225 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qafter_handle: LispObject =
    crate::lisp::LispObject(226 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qafter_insert_file_set_coding: LispObject =
    crate::lisp::LispObject(227 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qafter_string: LispObject =
    crate::lisp::LispObject(228 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qalist: LispObject =
    crate::lisp::LispObject(229 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qall_events: LispObject =
    crate::lisp::LispObject(230 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qalpha: LispObject =
    crate::lisp::LispObject(231 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qand_optional: LispObject =
    crate::lisp::LispObject(232 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qand_rest: LispObject =
    crate::lisp::LispObject(233 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qappend: LispObject =
    crate::lisp::LispObject(234 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qapply: LispObject =
    crate::lisp::LispObject(235 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qargs: LispObject =
    crate::lisp::LispObject(236 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qargs_out_of_range: LispObject =
    crate::lisp::LispObject(237 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qarith_error: LispObject =
    crate::lisp::LispObject(238 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qarray: LispObject =
    crate::lisp::LispObject(239 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qarrayp: LispObject =
    crate::lisp::LispObject(240 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qarrow: LispObject =
    crate::lisp::LispObject(241 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qascii: LispObject =
    crate::lisp::LispObject(242 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qascii_0: LispObject =
    crate::lisp::LispObject(243 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qascii_character: LispObject =
    crate::lisp::LispObject(244 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qassume: LispObject =
    crate::lisp::LispObject(245 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qattrib: LispObject =
    crate::lisp::LispObject(246 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qauto_composed: LispObject =
    crate::lisp::LispObject(247 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qauto_fill_chars: LispObject =
    crate::lisp::LispObject(248 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qauto_hscroll_mode: LispObject =
    crate::lisp::LispObject(249 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qauto_lower: LispObject =
    crate::lisp::LispObject(250 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qauto_raise: LispObject =
    crate::lisp::LispObject(251 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qauto_save_coding: LispObject =
    crate::lisp::LispObject(252 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qautoload: LispObject =
    crate::lisp::LispObject(253 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qbackground_color: LispObject =
    crate::lisp::LispObject(254 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qbackground_mode: LispObject =
    crate::lisp::LispObject(255 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qbackquote: LispObject =
    crate::lisp::LispObject(256 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qbar: LispObject =
    crate::lisp::LispObject(257 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qbefore_change_functions: LispObject =
    crate::lisp::LispObject(258 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qbefore_handle: LispObject =
    crate::lisp::LispObject(259 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qbefore_string: LispObject =
    crate::lisp::LispObject(260 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qbeginning_of_buffer: LispObject =
    crate::lisp::LispObject(261 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qbelow: LispObject =
    crate::lisp::LispObject(262 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qbelow_handle: LispObject =
    crate::lisp::LispObject(263 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qbig: LispObject =
    crate::lisp::LispObject(264 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qbig5: LispObject =
    crate::lisp::LispObject(265 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qbitmap_spec_p: LispObject =
    crate::lisp::LispObject(266 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qbold: LispObject =
    crate::lisp::LispObject(267 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qbool_vector: LispObject =
    crate::lisp::LispObject(268 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qbool_vector_p: LispObject =
    crate::lisp::LispObject(269 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qborder: LispObject =
    crate::lisp::LispObject(270 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qborder_color: LispObject =
    crate::lisp::LispObject(271 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qborder_width: LispObject =
    crate::lisp::LispObject(272 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qboth: LispObject =
    crate::lisp::LispObject(273 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qboth_horiz: LispObject =
    crate::lisp::LispObject(274 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qbottom: LispObject =
    crate::lisp::LispObject(275 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qbottom_divider: LispObject =
    crate::lisp::LispObject(276 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qbottom_divider_width: LispObject =
    crate::lisp::LispObject(277 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qbottom_edge: LispObject =
    crate::lisp::LispObject(278 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qbottom_left_corner: LispObject =
    crate::lisp::LispObject(279 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qbottom_right_corner: LispObject =
    crate::lisp::LispObject(280 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qboundary: LispObject =
    crate::lisp::LispObject(281 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qbounds: LispObject =
    crate::lisp::LispObject(282 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qbox: LispObject =
    crate::lisp::LispObject(283 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qbuffer: LispObject =
    crate::lisp::LispObject(284 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qbuffer_access_fontify_functions: LispObject =
    crate::lisp::LispObject(285 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qbuffer_file_coding_system: LispObject =
    crate::lisp::LispObject(286 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qbuffer_list: LispObject =
    crate::lisp::LispObject(287 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qbuffer_list_update_hook: LispObject =
    crate::lisp::LispObject(288 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qbuffer_name_history: LispObject =
    crate::lisp::LispObject(289 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qbuffer_or_string_p: LispObject =
    crate::lisp::LispObject(290 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qbuffer_position: LispObject =
    crate::lisp::LispObject(291 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qbuffer_predicate: LispObject =
    crate::lisp::LispObject(292 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qbuffer_read_only: LispObject =
    crate::lisp::LispObject(293 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qbufferp: LispObject =
    crate::lisp::LispObject(294 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qbuffers: LispObject =
    crate::lisp::LispObject(295 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qburied_buffer_list: LispObject =
    crate::lisp::LispObject(296 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qbyte_code_function_p: LispObject =
    crate::lisp::LispObject(297 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qbyte_code_meter: LispObject =
    crate::lisp::LispObject(298 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qbyte_run_unescaped_character_literals_warning: LispObject =
    crate::lisp::LispObject(299 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qc: LispObject =
    crate::lisp::LispObject(300 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcall: LispObject =
    crate::lisp::LispObject(301 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcall_process: LispObject =
    crate::lisp::LispObject(302 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcall_process_region: LispObject =
    crate::lisp::LispObject(303 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcallref: LispObject =
    crate::lisp::LispObject(304 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcar: LispObject =
    crate::lisp::LispObject(305 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcar_less_than_car: LispObject =
    crate::lisp::LispObject(306 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcase_fold_search: LispObject =
    crate::lisp::LispObject(307 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcase_table: LispObject =
    crate::lisp::LispObject(308 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcase_table_p: LispObject =
    crate::lisp::LispObject(309 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcatcher: LispObject =
    crate::lisp::LispObject(310 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcategory: LispObject =
    crate::lisp::LispObject(311 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcategory_table: LispObject =
    crate::lisp::LispObject(312 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcategory_table_p: LispObject =
    crate::lisp::LispObject(313 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcategoryp: LispObject =
    crate::lisp::LispObject(314 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcategorysetp: LispObject =
    crate::lisp::LispObject(315 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qccl: LispObject =
    crate::lisp::LispObject(316 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qccl_program_idx: LispObject =
    crate::lisp::LispObject(317 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcclp: LispObject =
    crate::lisp::LispObject(318 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcdr: LispObject =
    crate::lisp::LispObject(319 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qceiling: LispObject =
    crate::lisp::LispObject(320 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcenter: LispObject =
    crate::lisp::LispObject(321 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qchange_frame_size: LispObject =
    crate::lisp::LispObject(322 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qchange_major_mode_hook: LispObject =
    crate::lisp::LispObject(323 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qchar_code_property_table: LispObject =
    crate::lisp::LispObject(324 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qchar_from_name: LispObject =
    crate::lisp::LispObject(325 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qchar_or_string_p: LispObject =
    crate::lisp::LispObject(326 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qchar_script_table: LispObject =
    crate::lisp::LispObject(327 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qchar_table: LispObject =
    crate::lisp::LispObject(328 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qchar_table_extra_slots: LispObject =
    crate::lisp::LispObject(329 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qchar_table_p: LispObject =
    crate::lisp::LispObject(330 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcharacterp: LispObject =
    crate::lisp::LispObject(331 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcharset: LispObject =
    crate::lisp::LispObject(332 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcharsetp: LispObject =
    crate::lisp::LispObject(333 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qchild_frame_border: LispObject =
    crate::lisp::LispObject(334 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qchild_frame_border_width: LispObject =
    crate::lisp::LispObject(335 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qchoice: LispObject =
    crate::lisp::LispObject(336 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcircle: LispObject =
    crate::lisp::LispObject(337 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcircular_list: LispObject =
    crate::lisp::LispObject(338 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qclone_of: LispObject =
    crate::lisp::LispObject(339 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qclose: LispObject =
    crate::lisp::LispObject(340 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qclose_nowrite: LispObject =
    crate::lisp::LispObject(341 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qclose_tab: LispObject =
    crate::lisp::LispObject(342 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qclose_write: LispObject =
    crate::lisp::LispObject(343 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qclosed: LispObject =
    crate::lisp::LispObject(344 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qclosure: LispObject =
    crate::lisp::LispObject(345 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcmajflt: LispObject =
    crate::lisp::LispObject(346 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcminflt: LispObject =
    crate::lisp::LispObject(347 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcode_conversion_map: LispObject =
    crate::lisp::LispObject(348 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcode_conversion_map_id: LispObject =
    crate::lisp::LispObject(349 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcodeset: LispObject =
    crate::lisp::LispObject(350 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcoding_system_define_form: LispObject =
    crate::lisp::LispObject(351 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcoding_system_error: LispObject =
    crate::lisp::LispObject(352 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcoding_system_history: LispObject =
    crate::lisp::LispObject(353 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcoding_system_p: LispObject =
    crate::lisp::LispObject(354 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcolumns: LispObject =
    crate::lisp::LispObject(355 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcomm: LispObject =
    crate::lisp::LispObject(356 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcomma: LispObject =
    crate::lisp::LispObject(357 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcomma_at: LispObject =
    crate::lisp::LispObject(358 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcommand_debug_status: LispObject =
    crate::lisp::LispObject(359 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcommand_execute: LispObject =
    crate::lisp::LispObject(360 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcommand_modes: LispObject =
    crate::lisp::LispObject(361 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcommandp: LispObject =
    crate::lisp::LispObject(362 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcomment: LispObject =
    crate::lisp::LispObject(363 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcomment_end_can_be_escaped: LispObject =
    crate::lisp::LispObject(364 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcomp: LispObject =
    crate::lisp::LispObject(365 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcomp_debug: LispObject =
    crate::lisp::LispObject(366 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcomp_libgccjit_reproducer: LispObject =
    crate::lisp::LispObject(367 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcomp_maybe_gc_or_quit: LispObject =
    crate::lisp::LispObject(368 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcomp_mvar: LispObject =
    crate::lisp::LispObject(369 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcomp_native_driver_options: LispObject =
    crate::lisp::LispObject(370 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcomp_speed: LispObject =
    crate::lisp::LispObject(371 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcomp_subr_trampoline_install: LispObject =
    crate::lisp::LispObject(372 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcomp_warning_on_missing_source: LispObject =
    crate::lisp::LispObject(373 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcompiled_function: LispObject =
    crate::lisp::LispObject(374 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcompletion_ignore_case: LispObject =
    crate::lisp::LispObject(375 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcomposition: LispObject =
    crate::lisp::LispObject(376 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcond_jump: LispObject =
    crate::lisp::LispObject(377 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcond_jump_narg_leq: LispObject =
    crate::lisp::LispObject(378 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcondition_case: LispObject =
    crate::lisp::LispObject(379 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcondition_variable: LispObject =
    crate::lisp::LispObject(380 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcondition_variable_p: LispObject =
    crate::lisp::LispObject(381 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qconfig_changed_event: LispObject =
    crate::lisp::LispObject(382 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qconnect: LispObject =
    crate::lisp::LispObject(383 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcons: LispObject =
    crate::lisp::LispObject(384 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qconses: LispObject =
    crate::lisp::LispObject(385 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qconsp: LispObject =
    crate::lisp::LispObject(386 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcontinuation: LispObject =
    crate::lisp::LispObject(387 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcopy_directory: LispObject =
    crate::lisp::LispObject(388 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcopy_file: LispObject =
    crate::lisp::LispObject(389 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcount: LispObject =
    crate::lisp::LispObject(390 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcreate: LispObject =
    crate::lisp::LispObject(391 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcrop: LispObject =
    crate::lisp::LispObject(392 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcstime: LispObject =
    crate::lisp::LispObject(393 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qctime: LispObject =
    crate::lisp::LispObject(394 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcurrent_input_method: LispObject =
    crate::lisp::LispObject(395 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcurrent_line: LispObject =
    crate::lisp::LispObject(396 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcurrent_load_list: LispObject =
    crate::lisp::LispObject(397 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcurrent_minibuffer_command: LispObject =
    crate::lisp::LispObject(398 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcursor: LispObject =
    crate::lisp::LispObject(399 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcursor_color: LispObject =
    crate::lisp::LispObject(400 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcursor_in_echo_area: LispObject =
    crate::lisp::LispObject(401 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcursor_type: LispObject =
    crate::lisp::LispObject(402 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcurve: LispObject =
    crate::lisp::LispObject(403 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcustom_variable_history: LispObject =
    crate::lisp::LispObject(404 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcustom_variable_p: LispObject =
    crate::lisp::LispObject(405 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcutime: LispObject =
    crate::lisp::LispObject(406 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcycle_sort_function: LispObject =
    crate::lisp::LispObject(407 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcyclic_function_indirection: LispObject =
    crate::lisp::LispObject(408 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qcyclic_variable_indirection: LispObject =
    crate::lisp::LispObject(409 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qd: LispObject =
    crate::lisp::LispObject(410 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qd_default: LispObject =
    crate::lisp::LispObject(411 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qd_ephemeral: LispObject =
    crate::lisp::LispObject(412 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qd_impure: LispObject =
    crate::lisp::LispObject(413 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdata: LispObject =
    crate::lisp::LispObject(414 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdatagram: LispObject =
    crate::lisp::LispObject(415 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdays: LispObject =
    crate::lisp::LispObject(416 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdbus_error: LispObject =
    crate::lisp::LispObject(417 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdbus_event: LispObject =
    crate::lisp::LispObject(418 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdbus_message_internal: LispObject =
    crate::lisp::LispObject(419 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdeactivate_mark: LispObject =
    crate::lisp::LispObject(420 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdebug: LispObject =
    crate::lisp::LispObject(421 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdecomposed_characters: LispObject =
    crate::lisp::LispObject(422 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdefalias_fset_function: LispObject =
    crate::lisp::LispObject(423 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdefault: LispObject =
    crate::lisp::LispObject(424 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdefault_directory: LispObject =
    crate::lisp::LispObject(425 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdeferred_action_function: LispObject =
    crate::lisp::LispObject(426 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdefun: LispObject =
    crate::lisp::LispObject(427 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdefvaralias: LispObject =
    crate::lisp::LispObject(428 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdelay: LispObject =
    crate::lisp::LispObject(429 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdelayed_warnings_hook: LispObject =
    crate::lisp::LispObject(430 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdelete: LispObject =
    crate::lisp::LispObject(431 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdelete_before: LispObject =
    crate::lisp::LispObject(432 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdelete_by_moving_to_trash: LispObject =
    crate::lisp::LispObject(433 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdelete_directory: LispObject =
    crate::lisp::LispObject(434 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdelete_file: LispObject =
    crate::lisp::LispObject(435 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdelete_frame: LispObject =
    crate::lisp::LispObject(436 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdelete_frame_functions: LispObject =
    crate::lisp::LispObject(437 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdelete_self: LispObject =
    crate::lisp::LispObject(438 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdelete_terminal_functions: LispObject =
    crate::lisp::LispObject(439 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdelete_window: LispObject =
    crate::lisp::LispObject(440 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdescribe_map_tree: LispObject =
    crate::lisp::LispObject(441 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdir_ok: LispObject =
    crate::lisp::LispObject(442 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdirect_call: LispObject =
    crate::lisp::LispObject(443 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdirect_callref: LispObject =
    crate::lisp::LispObject(444 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdirectory_file_name: LispObject =
    crate::lisp::LispObject(445 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdirectory_files: LispObject =
    crate::lisp::LispObject(446 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdirectory_files_and_attributes: LispObject =
    crate::lisp::LispObject(447 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdisable_eval: LispObject =
    crate::lisp::LispObject(448 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdisabled: LispObject =
    crate::lisp::LispObject(449 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdisplay: LispObject =
    crate::lisp::LispObject(450 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdisplay_buffer: LispObject =
    crate::lisp::LispObject(451 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdisplay_fill_column_indicator: LispObject =
    crate::lisp::LispObject(452 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdisplay_fill_column_indicator_character: LispObject =
    crate::lisp::LispObject(453 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdisplay_fill_column_indicator_column: LispObject =
    crate::lisp::LispObject(454 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdisplay_line_numbers: LispObject =
    crate::lisp::LispObject(455 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdisplay_line_numbers_disable: LispObject =
    crate::lisp::LispObject(456 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdisplay_line_numbers_offset: LispObject =
    crate::lisp::LispObject(457 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdisplay_line_numbers_widen: LispObject =
    crate::lisp::LispObject(458 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdisplay_line_numbers_width: LispObject =
    crate::lisp::LispObject(459 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdisplay_table: LispObject =
    crate::lisp::LispObject(460 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdisplay_type: LispObject =
    crate::lisp::LispObject(461 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdo_after_load_evaluation: LispObject =
    crate::lisp::LispObject(462 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdomain_error: LispObject =
    crate::lisp::LispObject(463 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdont_follow: LispObject =
    crate::lisp::LispObject(464 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdos: LispObject =
    crate::lisp::LispObject(465 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdown: LispObject =
    crate::lisp::LispObject(466 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdrag_internal_border: LispObject =
    crate::lisp::LispObject(467 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdrag_n_drop: LispObject =
    crate::lisp::LispObject(468 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdragging: LispObject =
    crate::lisp::LispObject(469 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdropping: LispObject =
    crate::lisp::LispObject(470 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdump_emacs_portable__sort_predicate: LispObject =
    crate::lisp::LispObject(471 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdump_emacs_portable__sort_predicate_copied: LispObject =
    crate::lisp::LispObject(472 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdump_file_name: LispObject =
    crate::lisp::LispObject(473 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qdumped_with_pdumper: LispObject =
    crate::lisp::LispObject(474 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qecho_area_clear_hook: LispObject =
    crate::lisp::LispObject(475 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qecho_keystrokes: LispObject =
    crate::lisp::LispObject(476 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qedge_detection: LispObject =
    crate::lisp::LispObject(477 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qegid: LispObject =
    crate::lisp::LispObject(478 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qeight_bit: LispObject =
    crate::lisp::LispObject(479 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qemacs: LispObject =
    crate::lisp::LispObject(480 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qemacs_mule: LispObject =
    crate::lisp::LispObject(481 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qemboss: LispObject =
    crate::lisp::LispObject(482 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qempty_box: LispObject =
    crate::lisp::LispObject(483 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qempty_line: LispObject =
    crate::lisp::LispObject(484 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qenable_recursive_minibuffers: LispObject =
    crate::lisp::LispObject(485 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qencode_time: LispObject =
    crate::lisp::LispObject(486 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qend_of_buffer: LispObject =
    crate::lisp::LispObject(487 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qend_of_file: LispObject =
    crate::lisp::LispObject(488 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qend_scroll: LispObject =
    crate::lisp::LispObject(489 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qend_session: LispObject =
    crate::lisp::LispObject(490 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qentry: LispObject =
    crate::lisp::LispObject(491 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qeq: LispObject =
    crate::lisp::LispObject(492 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qeql: LispObject =
    crate::lisp::LispObject(493 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qequal: LispObject =
    crate::lisp::LispObject(494 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qerror_conditions: LispObject =
    crate::lisp::LispObject(495 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qerror_message: LispObject =
    crate::lisp::LispObject(496 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qescape_glyph: LispObject =
    crate::lisp::LispObject(497 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qetime: LispObject =
    crate::lisp::LispObject(498 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qeuid: LispObject =
    crate::lisp::LispObject(499 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qeval: LispObject =
    crate::lisp::LispObject(500 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qeval_buffer_list: LispObject =
    crate::lisp::LispObject(501 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qeval_expression: LispObject =
    crate::lisp::LispObject(502 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qevaporate: LispObject =
    crate::lisp::LispObject(503 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qeven: LispObject =
    crate::lisp::LispObject(504 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qevent_kind: LispObject =
    crate::lisp::LispObject(505 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qevent_symbol_element_mask: LispObject =
    crate::lisp::LispObject(506 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qevent_symbol_elements: LispObject =
    crate::lisp::LispObject(507 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qexcl: LispObject =
    crate::lisp::LispObject(508 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qexit: LispObject =
    crate::lisp::LispObject(509 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qexpand_abbrev: LispObject =
    crate::lisp::LispObject(510 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qexpand_file_name: LispObject =
    crate::lisp::LispObject(511 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qexplicit: LispObject =
    crate::lisp::LispObject(512 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qexplicit_name: LispObject =
    crate::lisp::LispObject(513 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qextension_data: LispObject =
    crate::lisp::LispObject(514 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qexternal_border_size: LispObject =
    crate::lisp::LispObject(515 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qexternal_debugging_output: LispObject =
    crate::lisp::LispObject(516 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qextra_bold: LispObject =
    crate::lisp::LispObject(517 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qextra_light: LispObject =
    crate::lisp::LispObject(518 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qface: LispObject =
    crate::lisp::LispObject(519 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qface_alias: LispObject =
    crate::lisp::LispObject(520 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qface_no_inherit: LispObject =
    crate::lisp::LispObject(521 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qface_remapping_alist: LispObject =
    crate::lisp::LispObject(522 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qface_set_after_frame_default: LispObject =
    crate::lisp::LispObject(523 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfailed: LispObject =
    crate::lisp::LispObject(524 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfboundp: LispObject =
    crate::lisp::LispObject(525 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfeatures: LispObject =
    crate::lisp::LispObject(526 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfetch_handler: LispObject =
    crate::lisp::LispObject(527 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfield: LispObject =
    crate::lisp::LispObject(528 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfile_accessible_directory_p: LispObject =
    crate::lisp::LispObject(529 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfile_acl: LispObject =
    crate::lisp::LispObject(530 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfile_already_exists: LispObject =
    crate::lisp::LispObject(531 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfile_attributes: LispObject =
    crate::lisp::LispObject(532 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfile_attributes_lessp: LispObject =
    crate::lisp::LispObject(533 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfile_date_error: LispObject =
    crate::lisp::LispObject(534 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfile_directory_p: LispObject =
    crate::lisp::LispObject(535 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfile_error: LispObject =
    crate::lisp::LispObject(536 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfile_executable_p: LispObject =
    crate::lisp::LispObject(537 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfile_exists_p: LispObject =
    crate::lisp::LispObject(538 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfile_missing: LispObject =
    crate::lisp::LispObject(539 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfile_modes: LispObject =
    crate::lisp::LispObject(540 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfile_name_all_completions: LispObject =
    crate::lisp::LispObject(541 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfile_name_as_directory: LispObject =
    crate::lisp::LispObject(542 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfile_name_case_insensitive_p: LispObject =
    crate::lisp::LispObject(543 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfile_name_completion: LispObject =
    crate::lisp::LispObject(544 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfile_name_directory: LispObject =
    crate::lisp::LispObject(545 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfile_name_handler_alist: LispObject =
    crate::lisp::LispObject(546 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfile_name_history: LispObject =
    crate::lisp::LispObject(547 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfile_name_nondirectory: LispObject =
    crate::lisp::LispObject(548 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfile_newer_than_file_p: LispObject =
    crate::lisp::LispObject(549 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfile_notify: LispObject =
    crate::lisp::LispObject(550 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfile_notify_error: LispObject =
    crate::lisp::LispObject(551 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfile_readable_p: LispObject =
    crate::lisp::LispObject(552 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfile_regular_p: LispObject =
    crate::lisp::LispObject(553 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfile_selinux_context: LispObject =
    crate::lisp::LispObject(554 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfile_symlink_p: LispObject =
    crate::lisp::LispObject(555 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfile_system_info: LispObject =
    crate::lisp::LispObject(556 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfile_writable_p: LispObject =
    crate::lisp::LispObject(557 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfilenamep: LispObject =
    crate::lisp::LispObject(558 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfill_column_indicator: LispObject =
    crate::lisp::LispObject(559 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfinalizer: LispObject =
    crate::lisp::LispObject(560 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfirst_change_hook: LispObject =
    crate::lisp::LispObject(561 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfixnum: LispObject =
    crate::lisp::LispObject(562 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfixnump: LispObject =
    crate::lisp::LispObject(563 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qflat_button: LispObject =
    crate::lisp::LispObject(564 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfloat: LispObject =
    crate::lisp::LispObject(565 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfloatp: LispObject =
    crate::lisp::LispObject(566 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfloats: LispObject =
    crate::lisp::LispObject(567 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfloor: LispObject =
    crate::lisp::LispObject(568 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfocus_in: LispObject =
    crate::lisp::LispObject(569 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfocus_out: LispObject =
    crate::lisp::LispObject(570 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfont: LispObject =
    crate::lisp::LispObject(571 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfont_backend: LispObject =
    crate::lisp::LispObject(572 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfont_driver_superseded_by: LispObject =
    crate::lisp::LispObject(573 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfont_entity: LispObject =
    crate::lisp::LispObject(574 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfont_lock_face: LispObject =
    crate::lisp::LispObject(575 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfont_object: LispObject =
    crate::lisp::LispObject(576 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfont_spec: LispObject =
    crate::lisp::LispObject(577 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfontification_functions: LispObject =
    crate::lisp::LispObject(578 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfontified: LispObject =
    crate::lisp::LispObject(579 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfontset: LispObject =
    crate::lisp::LispObject(580 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfontset_info: LispObject =
    crate::lisp::LispObject(581 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qforeground_color: LispObject =
    crate::lisp::LispObject(582 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qformat_annotate_function: LispObject =
    crate::lisp::LispObject(583 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qformat_decode: LispObject =
    crate::lisp::LispObject(584 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfraction: LispObject =
    crate::lisp::LispObject(585 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qframe: LispObject =
    crate::lisp::LispObject(586 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qframe_edges: LispObject =
    crate::lisp::LispObject(587 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qframe_inhibit_resize: LispObject =
    crate::lisp::LispObject(588 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qframe_live_p: LispObject =
    crate::lisp::LispObject(589 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qframe_monitor_attributes: LispObject =
    crate::lisp::LispObject(590 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qframe_set_background_mode: LispObject =
    crate::lisp::LispObject(591 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qframe_windows_min_size: LispObject =
    crate::lisp::LispObject(592 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qframep: LispObject =
    crate::lisp::LispObject(593 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qframes: LispObject =
    crate::lisp::LispObject(594 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfree_frame_menubar_1: LispObject =
    crate::lisp::LispObject(595 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfree_frame_menubar_2: LispObject =
    crate::lisp::LispObject(596 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfree_frame_tab_bar: LispObject =
    crate::lisp::LispObject(597 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfree_frame_tool_bar: LispObject =
    crate::lisp::LispObject(598 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfringe: LispObject =
    crate::lisp::LispObject(599 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfront_sticky: LispObject =
    crate::lisp::LispObject(600 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfullboth: LispObject =
    crate::lisp::LispObject(601 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfullheight: LispObject =
    crate::lisp::LispObject(602 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfullscreen: LispObject =
    crate::lisp::LispObject(603 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfullwidth: LispObject =
    crate::lisp::LispObject(604 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfuncall: LispObject =
    crate::lisp::LispObject(605 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfuncall_interactively: LispObject =
    crate::lisp::LispObject(606 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfunction: LispObject =
    crate::lisp::LispObject(607 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfunction_documentation: LispObject =
    crate::lisp::LispObject(608 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfunction_key: LispObject =
    crate::lisp::LispObject(609 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qfundamental_mode: LispObject =
    crate::lisp::LispObject(610 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qgc_cons_percentage: LispObject =
    crate::lisp::LispObject(611 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qgc_cons_threshold: LispObject =
    crate::lisp::LispObject(612 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qgccjit: LispObject =
    crate::lisp::LispObject(613 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qgdiplus: LispObject =
    crate::lisp::LispObject(614 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qgdk_pixbuf: LispObject =
    crate::lisp::LispObject(615 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qgeometry: LispObject =
    crate::lisp::LispObject(616 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qget_buffer_window_list: LispObject =
    crate::lisp::LispObject(617 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qget_emacs_mule_file_char: LispObject =
    crate::lisp::LispObject(618 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qget_file_buffer: LispObject =
    crate::lisp::LispObject(619 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qget_file_char: LispObject =
    crate::lisp::LispObject(620 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qget_mru_window: LispObject =
    crate::lisp::LispObject(621 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qgif: LispObject =
    crate::lisp::LispObject(622 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qgio: LispObject =
    crate::lisp::LispObject(623 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qglib: LispObject =
    crate::lisp::LispObject(624 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qglyphless_char: LispObject =
    crate::lisp::LispObject(625 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qglyphless_char_display: LispObject =
    crate::lisp::LispObject(626 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qgnutls_anon: LispObject =
    crate::lisp::LispObject(627 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qgnutls_code: LispObject =
    crate::lisp::LispObject(628 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qgnutls_e_again: LispObject =
    crate::lisp::LispObject(629 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qgnutls_e_interrupted: LispObject =
    crate::lisp::LispObject(630 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qgnutls_e_invalid_session: LispObject =
    crate::lisp::LispObject(631 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qgnutls_e_not_ready_for_handshake: LispObject =
    crate::lisp::LispObject(632 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qgnutls_type_cipher: LispObject =
    crate::lisp::LispObject(633 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qgnutls_type_digest_algorithm: LispObject =
    crate::lisp::LispObject(634 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qgnutls_type_mac_algorithm: LispObject =
    crate::lisp::LispObject(635 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qgnutls_x509pki: LispObject =
    crate::lisp::LispObject(636 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qgobject: LispObject =
    crate::lisp::LispObject(637 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qgrave: LispObject =
    crate::lisp::LispObject(638 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qgroup: LispObject =
    crate::lisp::LispObject(639 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qgrow_only: LispObject =
    crate::lisp::LispObject(640 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qgui_set_selection: LispObject =
    crate::lisp::LispObject(641 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qhand: LispObject =
    crate::lisp::LispObject(642 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qhandle: LispObject =
    crate::lisp::LispObject(643 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qhandle_select_window: LispObject =
    crate::lisp::LispObject(644 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qhandle_shift_selection: LispObject =
    crate::lisp::LispObject(645 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qhandle_switch_frame: LispObject =
    crate::lisp::LispObject(646 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qhash_table: LispObject =
    crate::lisp::LispObject(647 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qhash_table_p: LispObject =
    crate::lisp::LispObject(648 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qhash_table_test: LispObject =
    crate::lisp::LispObject(649 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qhbar: LispObject =
    crate::lisp::LispObject(650 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qheader_line: LispObject =
    crate::lisp::LispObject(651 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qheader_line_format: LispObject =
    crate::lisp::LispObject(652 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qheap: LispObject =
    crate::lisp::LispObject(653 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qheight: LispObject =
    crate::lisp::LispObject(654 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qheight_only: LispObject =
    crate::lisp::LispObject(655 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qhelp_echo: LispObject =
    crate::lisp::LispObject(656 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qhelp_echo_inhibit_substitution: LispObject =
    crate::lisp::LispObject(657 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qhelp_form_show: LispObject =
    crate::lisp::LispObject(658 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qhelp_key_binding: LispObject =
    crate::lisp::LispObject(659 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qhelper_save_restriction: LispObject =
    crate::lisp::LispObject(660 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qhelper_unbind_n: LispObject =
    crate::lisp::LispObject(661 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qhelper_unwind_protect: LispObject =
    crate::lisp::LispObject(662 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qheuristic: LispObject =
    crate::lisp::LispObject(663 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qhex_code: LispObject =
    crate::lisp::LispObject(664 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qhollow: LispObject =
    crate::lisp::LispObject(665 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qhollow_small: LispObject =
    crate::lisp::LispObject(666 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qhorizontal_handle: LispObject =
    crate::lisp::LispObject(667 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qhorizontal_scroll_bar: LispObject =
    crate::lisp::LispObject(668 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qhorizontal_scroll_bars: LispObject =
    crate::lisp::LispObject(669 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qhw: LispObject =
    crate::lisp::LispObject(670 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qicon: LispObject =
    crate::lisp::LispObject(671 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qicon_left: LispObject =
    crate::lisp::LispObject(672 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qicon_name: LispObject =
    crate::lisp::LispObject(673 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qicon_top: LispObject =
    crate::lisp::LispObject(674 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qicon_type: LispObject =
    crate::lisp::LispObject(675 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qiconify_frame: LispObject =
    crate::lisp::LispObject(676 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qiconify_top_level: LispObject =
    crate::lisp::LispObject(677 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qidentity: LispObject =
    crate::lisp::LispObject(678 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qif: LispObject =
    crate::lisp::LispObject(679 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qignored: LispObject =
    crate::lisp::LispObject(680 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qimage: LispObject =
    crate::lisp::LispObject(681 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qimagemagick: LispObject =
    crate::lisp::LispObject(682 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qinc_args: LispObject =
    crate::lisp::LispObject(683 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qinhibit_changing_match_data: LispObject =
    crate::lisp::LispObject(684 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qinhibit_debugger: LispObject =
    crate::lisp::LispObject(685 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qinhibit_double_buffering: LispObject =
    crate::lisp::LispObject(686 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qinhibit_eval_during_redisplay: LispObject =
    crate::lisp::LispObject(687 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qinhibit_file_name_operation: LispObject =
    crate::lisp::LispObject(688 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qinhibit_free_realized_faces: LispObject =
    crate::lisp::LispObject(689 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qinhibit_menubar_update: LispObject =
    crate::lisp::LispObject(690 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qinhibit_modification_hooks: LispObject =
    crate::lisp::LispObject(691 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qinhibit_point_motion_hooks: LispObject =
    crate::lisp::LispObject(692 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qinhibit_quit: LispObject =
    crate::lisp::LispObject(693 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qinhibit_read_only: LispObject =
    crate::lisp::LispObject(694 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qinhibit_redisplay: LispObject =
    crate::lisp::LispObject(695 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qinhibited_interaction: LispObject =
    crate::lisp::LispObject(696 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qinner_edges: LispObject =
    crate::lisp::LispObject(697 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qinput_method_exit_on_first_char: LispObject =
    crate::lisp::LispObject(698 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qinput_method_use_echo_area: LispObject =
    crate::lisp::LispObject(699 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qinsert_behind_hooks: LispObject =
    crate::lisp::LispObject(700 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qinsert_file_contents: LispObject =
    crate::lisp::LispObject(701 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qinsert_in_front_hooks: LispObject =
    crate::lisp::LispObject(702 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qinsufficient_source: LispObject =
    crate::lisp::LispObject(703 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qintangible: LispObject =
    crate::lisp::LispObject(704 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qinteger: LispObject =
    crate::lisp::LispObject(705 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qinteger_or_marker_p: LispObject =
    crate::lisp::LispObject(706 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qintegerp: LispObject =
    crate::lisp::LispObject(707 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qinteractive: LispObject =
    crate::lisp::LispObject(708 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qinteractive_form: LispObject =
    crate::lisp::LispObject(709 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qinternal__syntax_propertize: LispObject =
    crate::lisp::LispObject(710 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qinternal_auto_fill: LispObject =
    crate::lisp::LispObject(711 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qinternal_border: LispObject =
    crate::lisp::LispObject(712 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qinternal_border_width: LispObject =
    crate::lisp::LispObject(713 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qinternal_default_interrupt_process: LispObject =
    crate::lisp::LispObject(714 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qinternal_default_process_filter: LispObject =
    crate::lisp::LispObject(715 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qinternal_default_process_sentinel: LispObject =
    crate::lisp::LispObject(716 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qinternal_echo_keystrokes_prefix: LispObject =
    crate::lisp::LispObject(717 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qinternal_interpreter_environment: LispObject =
    crate::lisp::LispObject(718 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qinternal_when_entered_debugger: LispObject =
    crate::lisp::LispObject(719 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qinterrupt_process_functions: LispObject =
    crate::lisp::LispObject(720 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qinterrupted: LispObject =
    crate::lisp::LispObject(721 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qintervals: LispObject =
    crate::lisp::LispObject(722 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qinvalid_arity: LispObject =
    crate::lisp::LispObject(723 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qinvalid_function: LispObject =
    crate::lisp::LispObject(724 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qinvalid_read_syntax: LispObject =
    crate::lisp::LispObject(725 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qinvalid_regexp: LispObject =
    crate::lisp::LispObject(726 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qinvalid_source: LispObject =
    crate::lisp::LispObject(727 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qinvisible: LispObject =
    crate::lisp::LispObject(728 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qipv4: LispObject =
    crate::lisp::LispObject(729 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qipv6: LispObject =
    crate::lisp::LispObject(730 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qisdir: LispObject =
    crate::lisp::LispObject(731 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qiso10646_1: LispObject =
    crate::lisp::LispObject(732 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qiso8859_1: LispObject =
    crate::lisp::LispObject(733 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qiso_2022: LispObject =
    crate::lisp::LispObject(734 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qiso_8859_1: LispObject =
    crate::lisp::LispObject(735 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qitalic: LispObject =
    crate::lisp::LispObject(736 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qiv_auto: LispObject =
    crate::lisp::LispObject(737 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qja: LispObject =
    crate::lisp::LispObject(738 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qjpeg: LispObject =
    crate::lisp::LispObject(739 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qjs__clear: LispObject =
    crate::lisp::LispObject(740 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qjs__reenter: LispObject =
    crate::lisp::LispObject(741 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qjs_error: LispObject =
    crate::lisp::LispObject(742 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qjs_lisp_error: LispObject =
    crate::lisp::LispObject(743 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qjs_tick_event_loop: LispObject =
    crate::lisp::LispObject(744 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qjson_end_of_file: LispObject =
    crate::lisp::LispObject(745 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qjson_error: LispObject =
    crate::lisp::LispObject(746 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qjson_object_too_deep: LispObject =
    crate::lisp::LispObject(747 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qjson_out_of_memory: LispObject =
    crate::lisp::LispObject(748 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qjson_parse_error: LispObject =
    crate::lisp::LispObject(749 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qjson_parse_string: LispObject =
    crate::lisp::LispObject(750 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qjson_serialize: LispObject =
    crate::lisp::LispObject(751 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qjson_trailing_content: LispObject =
    crate::lisp::LispObject(752 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qjson_value_p: LispObject =
    crate::lisp::LispObject(753 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qjump: LispObject =
    crate::lisp::LispObject(754 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qkbd_macro_termination_hook: LispObject =
    crate::lisp::LispObject(755 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qkeep_ratio: LispObject =
    crate::lisp::LispObject(756 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qkey: LispObject =
    crate::lisp::LispObject(757 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qkey_and_value: LispObject =
    crate::lisp::LispObject(758 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qkey_or_value: LispObject =
    crate::lisp::LispObject(759 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qkeymap: LispObject =
    crate::lisp::LispObject(760 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qkeymap_canonicalize: LispObject =
    crate::lisp::LispObject(761 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qkeymapp: LispObject =
    crate::lisp::LispObject(762 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qkill_buffer_hook: LispObject =
    crate::lisp::LispObject(763 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qkill_buffer_query_functions: LispObject =
    crate::lisp::LispObject(764 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qkill_emacs: LispObject =
    crate::lisp::LispObject(765 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qkill_emacs_hook: LispObject =
    crate::lisp::LispObject(766 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qkill_forward_chars: LispObject =
    crate::lisp::LispObject(767 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qko: LispObject =
    crate::lisp::LispObject(768 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qlambda_fixup: LispObject =
    crate::lisp::LispObject(769 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qlanguage_change: LispObject =
    crate::lisp::LispObject(770 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qlaplace: LispObject =
    crate::lisp::LispObject(771 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qlast_arrow_position: LispObject =
    crate::lisp::LispObject(772 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qlast_arrow_string: LispObject =
    crate::lisp::LispObject(773 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qlast_nonmenu_event: LispObject =
    crate::lisp::LispObject(774 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qlate: LispObject =
    crate::lisp::LispObject(775 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qlatin: LispObject =
    crate::lisp::LispObject(776 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qleft: LispObject =
    crate::lisp::LispObject(777 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qleft_edge: LispObject =
    crate::lisp::LispObject(778 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qleft_fringe: LispObject =
    crate::lisp::LispObject(779 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qleft_margin: LispObject =
    crate::lisp::LispObject(780 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qleft_only: LispObject =
    crate::lisp::LispObject(781 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qleft_to_right: LispObject =
    crate::lisp::LispObject(782 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qleftmost: LispObject =
    crate::lisp::LispObject(783 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qlet: LispObject =
    crate::lisp::LispObject(784 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qletx: LispObject =
    crate::lisp::LispObject(785 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qlexical_binding: LispObject =
    crate::lisp::LispObject(786 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qlibgif_version: LispObject =
    crate::lisp::LispObject(787 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qlibjpeg_version: LispObject =
    crate::lisp::LispObject(788 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qlibpng_version: LispObject =
    crate::lisp::LispObject(789 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qlight: LispObject =
    crate::lisp::LispObject(790 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qline: LispObject =
    crate::lisp::LispObject(791 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qline_height: LispObject =
    crate::lisp::LispObject(792 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qline_number: LispObject =
    crate::lisp::LispObject(793 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qline_number_current_line: LispObject =
    crate::lisp::LispObject(794 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qline_number_major_tick: LispObject =
    crate::lisp::LispObject(795 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qline_number_minor_tick: LispObject =
    crate::lisp::LispObject(796 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qline_prefix: LispObject =
    crate::lisp::LispObject(797 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qline_spacing: LispObject =
    crate::lisp::LispObject(798 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qlist: LispObject =
    crate::lisp::LispObject(799 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qlist_or_vector_p: LispObject =
    crate::lisp::LispObject(800 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qlisten: LispObject =
    crate::lisp::LispObject(801 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qlistp: LispObject =
    crate::lisp::LispObject(802 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qlittle: LispObject =
    crate::lisp::LispObject(803 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qload: LispObject =
    crate::lisp::LispObject(804 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qload_file_name: LispObject =
    crate::lisp::LispObject(805 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qload_force_doc_strings: LispObject =
    crate::lisp::LispObject(806 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qload_in_progress: LispObject =
    crate::lisp::LispObject(807 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qload_time: LispObject =
    crate::lisp::LispObject(808 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qload_true_file_name: LispObject =
    crate::lisp::LispObject(809 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qlocal: LispObject =
    crate::lisp::LispObject(810 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qlocal_map: LispObject =
    crate::lisp::LispObject(811 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qlong: LispObject =
    crate::lisp::LispObject(812 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qlread_unescaped_character_literals: LispObject =
    crate::lisp::LispObject(813 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qm: LispObject =
    crate::lisp::LispObject(814 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmac: LispObject =
    crate::lisp::LispObject(815 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmacro: LispObject =
    crate::lisp::LispObject(816 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmajflt: LispObject =
    crate::lisp::LispObject(817 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmake_cursor_line_fully_visible: LispObject =
    crate::lisp::LispObject(818 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmake_directory: LispObject =
    crate::lisp::LispObject(819 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmake_directory_internal: LispObject =
    crate::lisp::LispObject(820 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmake_frame_visible: LispObject =
    crate::lisp::LispObject(821 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmake_invisible: LispObject =
    crate::lisp::LispObject(822 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmake_process: LispObject =
    crate::lisp::LispObject(823 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmake_symbolic_link: LispObject =
    crate::lisp::LispObject(824 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmakunbound: LispObject =
    crate::lisp::LispObject(825 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmany: LispObject =
    crate::lisp::LispObject(826 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmargin: LispObject =
    crate::lisp::LispObject(827 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmark_for_redisplay: LispObject =
    crate::lisp::LispObject(828 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmark_inactive: LispObject =
    crate::lisp::LispObject(829 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmarker: LispObject =
    crate::lisp::LispObject(830 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmarkerp: LispObject =
    crate::lisp::LispObject(831 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmaximized: LispObject =
    crate::lisp::LispObject(832 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmd5: LispObject =
    crate::lisp::LispObject(833 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmenu: LispObject =
    crate::lisp::LispObject(834 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmenu_bar: LispObject =
    crate::lisp::LispObject(835 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmenu_bar_external: LispObject =
    crate::lisp::LispObject(836 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmenu_bar_lines: LispObject =
    crate::lisp::LispObject(837 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmenu_bar_size: LispObject =
    crate::lisp::LispObject(838 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmenu_bar_update_hook: LispObject =
    crate::lisp::LispObject(839 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmenu_enable: LispObject =
    crate::lisp::LispObject(840 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmenu_item: LispObject =
    crate::lisp::LispObject(841 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmetadata: LispObject =
    crate::lisp::LispObject(842 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmin_height: LispObject =
    crate::lisp::LispObject(843 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmin_width: LispObject =
    crate::lisp::LispObject(844 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qminflt: LispObject =
    crate::lisp::LispObject(845 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qminibuffer: LispObject =
    crate::lisp::LispObject(846 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qminibuffer_completion_table: LispObject =
    crate::lisp::LispObject(847 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qminibuffer_default: LispObject =
    crate::lisp::LispObject(848 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qminibuffer_exit: LispObject =
    crate::lisp::LispObject(849 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qminibuffer_exit_hook: LispObject =
    crate::lisp::LispObject(850 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qminibuffer_follows_selected_frame: LispObject =
    crate::lisp::LispObject(851 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qminibuffer_history: LispObject =
    crate::lisp::LispObject(852 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qminibuffer_prompt: LispObject =
    crate::lisp::LispObject(853 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qminibuffer_setup_hook: LispObject =
    crate::lisp::LispObject(854 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qminus: LispObject =
    crate::lisp::LispObject(855 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmissing_module_init_function: LispObject =
    crate::lisp::LispObject(856 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmm_size: LispObject =
    crate::lisp::LispObject(857 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmode_class: LispObject =
    crate::lisp::LispObject(858 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmode_line: LispObject =
    crate::lisp::LispObject(859 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmode_line_default_help_echo: LispObject =
    crate::lisp::LispObject(860 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmode_line_format: LispObject =
    crate::lisp::LispObject(861 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmode_line_inactive: LispObject =
    crate::lisp::LispObject(862 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmodeline: LispObject =
    crate::lisp::LispObject(863 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmodification_hooks: LispObject =
    crate::lisp::LispObject(864 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmodifier_cache: LispObject =
    crate::lisp::LispObject(865 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmodify: LispObject =
    crate::lisp::LispObject(866 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmodule_function: LispObject =
    crate::lisp::LispObject(867 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmodule_function_p: LispObject =
    crate::lisp::LispObject(868 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmodule_init_failed: LispObject =
    crate::lisp::LispObject(869 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmodule_load_failed: LispObject =
    crate::lisp::LispObject(870 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmodule_not_gpl_compatible: LispObject =
    crate::lisp::LispObject(871 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmodule_open_failed: LispObject =
    crate::lisp::LispObject(872 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmonths: LispObject =
    crate::lisp::LispObject(873 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmouse: LispObject =
    crate::lisp::LispObject(874 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmouse_click: LispObject =
    crate::lisp::LispObject(875 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmouse_color: LispObject =
    crate::lisp::LispObject(876 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmouse_face: LispObject =
    crate::lisp::LispObject(877 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmouse_fixup_help_message: LispObject =
    crate::lisp::LispObject(878 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmouse_leave_buffer_hook: LispObject =
    crate::lisp::LispObject(879 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmouse_movement: LispObject =
    crate::lisp::LispObject(880 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmouse_wheel_frame: LispObject =
    crate::lisp::LispObject(881 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmove: LispObject =
    crate::lisp::LispObject(882 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmove_file_to_trash: LispObject =
    crate::lisp::LispObject(883 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmove_frame: LispObject =
    crate::lisp::LispObject(884 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmove_self: LispObject =
    crate::lisp::LispObject(885 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmoved_from: LispObject =
    crate::lisp::LispObject(886 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmoved_to: LispObject =
    crate::lisp::LispObject(887 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmutex: LispObject =
    crate::lisp::LispObject(888 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qmutexp: LispObject =
    crate::lisp::LispObject(889 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qname: LispObject =
    crate::lisp::LispObject(890 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qnative_comp_unit: LispObject =
    crate::lisp::LispObject(891 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qnative_compiler_error: LispObject =
    crate::lisp::LispObject(892 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qnative_edges: LispObject =
    crate::lisp::LispObject(893 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qnative_ice: LispObject =
    crate::lisp::LispObject(894 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qnative_image: LispObject =
    crate::lisp::LispObject(895 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qnative_lisp_file_inconsistent: LispObject =
    crate::lisp::LispObject(896 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qnative_lisp_load_failed: LispObject =
    crate::lisp::LispObject(897 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qnative_lisp_wrong_reloc: LispObject =
    crate::lisp::LispObject(898 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qnatnump: LispObject =
    crate::lisp::LispObject(899 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qnegate: LispObject =
    crate::lisp::LispObject(900 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qnetwork: LispObject =
    crate::lisp::LispObject(901 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qnice: LispObject =
    crate::lisp::LispObject(902 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qno_accept_focus: LispObject =
    crate::lisp::LispObject(903 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qno_catch: LispObject =
    crate::lisp::LispObject(904 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qno_conversion: LispObject =
    crate::lisp::LispObject(905 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qno_focus_on_map: LispObject =
    crate::lisp::LispObject(906 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qno_other_frame: LispObject =
    crate::lisp::LispObject(907 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qno_record: LispObject =
    crate::lisp::LispObject(908 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qno_special_glyphs: LispObject =
    crate::lisp::LispObject(909 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qnobreak_hyphen: LispObject =
    crate::lisp::LispObject(910 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qnobreak_space: LispObject =
    crate::lisp::LispObject(911 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qnoelisp: LispObject =
    crate::lisp::LispObject(912 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qnon_ascii: LispObject =
    crate::lisp::LispObject(913 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qnone: LispObject =
    crate::lisp::LispObject(914 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qnormal: LispObject =
    crate::lisp::LispObject(915 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qns: LispObject =
    crate::lisp::LispObject(916 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qns_appearance: LispObject =
    crate::lisp::LispObject(917 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qns_parse_geometry: LispObject =
    crate::lisp::LispObject(918 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qns_transparent_titlebar: LispObject =
    crate::lisp::LispObject(919 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qnsm_verify_connection: LispObject =
    crate::lisp::LispObject(920 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qnull: LispObject =
    crate::lisp::LispObject(921 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qnumber_or_marker_p: LispObject =
    crate::lisp::LispObject(922 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qnumberp: LispObject =
    crate::lisp::LispObject(923 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qobject: LispObject =
    crate::lisp::LispObject(924 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qoblique: LispObject =
    crate::lisp::LispObject(925 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qodd: LispObject =
    crate::lisp::LispObject(926 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qonly: LispObject =
    crate::lisp::LispObject(927 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qonlydir: LispObject =
    crate::lisp::LispObject(928 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qopen: LispObject =
    crate::lisp::LispObject(929 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qopen_network_stream: LispObject =
    crate::lisp::LispObject(930 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qopentype: LispObject =
    crate::lisp::LispObject(931 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qoperations: LispObject =
    crate::lisp::LispObject(932 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qouter_border_width: LispObject =
    crate::lisp::LispObject(933 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qouter_edges: LispObject =
    crate::lisp::LispObject(934 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qouter_position: LispObject =
    crate::lisp::LispObject(935 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qouter_size: LispObject =
    crate::lisp::LispObject(936 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qouter_window_id: LispObject =
    crate::lisp::LispObject(937 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qoverflow_error: LispObject =
    crate::lisp::LispObject(938 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qoverlay: LispObject =
    crate::lisp::LispObject(939 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qoverlay_arrow: LispObject =
    crate::lisp::LispObject(940 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qoverlay_arrow_bitmap: LispObject =
    crate::lisp::LispObject(941 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qoverlay_arrow_string: LispObject =
    crate::lisp::LispObject(942 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qoverlayp: LispObject =
    crate::lisp::LispObject(943 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qoverride_redirect: LispObject =
    crate::lisp::LispObject(944 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qoverriding_local_map: LispObject =
    crate::lisp::LispObject(945 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qoverriding_plist_environment: LispObject =
    crate::lisp::LispObject(946 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qoverriding_terminal_local_map: LispObject =
    crate::lisp::LispObject(947 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qoverwrite_mode: LispObject =
    crate::lisp::LispObject(948 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qoverwrite_mode_binary: LispObject =
    crate::lisp::LispObject(949 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qp: LispObject =
    crate::lisp::LispObject(950 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qpaper: LispObject =
    crate::lisp::LispObject(951 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qparent_frame: LispObject =
    crate::lisp::LispObject(952 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qparent_id: LispObject =
    crate::lisp::LispObject(953 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qpbm: LispObject =
    crate::lisp::LispObject(954 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qpc: LispObject =
    crate::lisp::LispObject(955 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qpcpu: LispObject =
    crate::lisp::LispObject(956 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qpermanent_local: LispObject =
    crate::lisp::LispObject(957 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qpermanent_local_hook: LispObject =
    crate::lisp::LispObject(958 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qpgrp: LispObject =
    crate::lisp::LispObject(959 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qphi: LispObject =
    crate::lisp::LispObject(960 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qpipe: LispObject =
    crate::lisp::LispObject(961 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qpipe_process_p: LispObject =
    crate::lisp::LispObject(962 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qplay_sound_functions: LispObject =
    crate::lisp::LispObject(963 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qplist: LispObject =
    crate::lisp::LispObject(964 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qplistp: LispObject =
    crate::lisp::LispObject(965 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qplus: LispObject =
    crate::lisp::LispObject(966 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qpmem: LispObject =
    crate::lisp::LispObject(967 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qpng: LispObject =
    crate::lisp::LispObject(968 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qpoint_entered: LispObject =
    crate::lisp::LispObject(969 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qpoint_left: LispObject =
    crate::lisp::LispObject(970 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qpointer: LispObject =
    crate::lisp::LispObject(971 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qpolling_period: LispObject =
    crate::lisp::LispObject(972 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qpoly: LispObject =
    crate::lisp::LispObject(973 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qpop_handler: LispObject =
    crate::lisp::LispObject(974 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qposition: LispObject =
    crate::lisp::LispObject(975 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qpost_command_hook: LispObject =
    crate::lisp::LispObject(976 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qpost_gc_hook: LispObject =
    crate::lisp::LispObject(977 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qpost_self_insert_hook: LispObject =
    crate::lisp::LispObject(978 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qpostscript: LispObject =
    crate::lisp::LispObject(979 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qppid: LispObject =
    crate::lisp::LispObject(980 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qpre_command_hook: LispObject =
    crate::lisp::LispObject(981 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qpressed_button: LispObject =
    crate::lisp::LispObject(982 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qpri: LispObject =
    crate::lisp::LispObject(983 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qprint_escape_multibyte: LispObject =
    crate::lisp::LispObject(984 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qprint_escape_nonascii: LispObject =
    crate::lisp::LispObject(985 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qpriority: LispObject =
    crate::lisp::LispObject(986 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qprocess: LispObject =
    crate::lisp::LispObject(987 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qprocessp: LispObject =
    crate::lisp::LispObject(988 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qprofiler_backtrace_equal: LispObject =
    crate::lisp::LispObject(989 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qprogn: LispObject =
    crate::lisp::LispObject(990 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qpropertize: LispObject =
    crate::lisp::LispObject(991 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qprotected_field: LispObject =
    crate::lisp::LispObject(992 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qprovide: LispObject =
    crate::lisp::LispObject(993 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qpty: LispObject =
    crate::lisp::LispObject(994 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qpure: LispObject =
    crate::lisp::LispObject(995 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qpurecopy: LispObject =
    crate::lisp::LispObject(996 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qpush_handler: LispObject =
    crate::lisp::LispObject(997 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qq_overflow: LispObject =
    crate::lisp::LispObject(998 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qquit: LispObject =
    crate::lisp::LispObject(999 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qquote: LispObject =
    crate::lisp::LispObject(1000 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qraise: LispObject =
    crate::lisp::LispObject(1001 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qrange: LispObject =
    crate::lisp::LispObject(1002 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qrange_error: LispObject =
    crate::lisp::LispObject(1003 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qratio: LispObject =
    crate::lisp::LispObject(1004 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qraw_text: LispObject =
    crate::lisp::LispObject(1005 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qread: LispObject =
    crate::lisp::LispObject(1006 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qread_char: LispObject =
    crate::lisp::LispObject(1007 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qread_number: LispObject =
    crate::lisp::LispObject(1008 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qread_only: LispObject =
    crate::lisp::LispObject(1009 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qreal: LispObject =
    crate::lisp::LispObject(1010 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qrear_nonsticky: LispObject =
    crate::lisp::LispObject(1011 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qrecompute_lucid_menubar: LispObject =
    crate::lisp::LispObject(1012 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qrecord: LispObject =
    crate::lisp::LispObject(1013 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qrecord_unwind_current_buffer: LispObject =
    crate::lisp::LispObject(1014 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qrecord_unwind_protect_excursion: LispObject =
    crate::lisp::LispObject(1015 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qrecord_window_buffer: LispObject =
    crate::lisp::LispObject(1016 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qrecordp: LispObject =
    crate::lisp::LispObject(1017 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qrect: LispObject =
    crate::lisp::LispObject(1018 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qredisplay_dont_pause: LispObject =
    crate::lisp::LispObject(1019 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qredisplay_end_trigger_functions: LispObject =
    crate::lisp::LispObject(1020 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qredisplay_internal_xC_functionx: LispObject =
    crate::lisp::LispObject(1021 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qrehash_size: LispObject =
    crate::lisp::LispObject(1022 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qrehash_threshold: LispObject =
    crate::lisp::LispObject(1023 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qrelative: LispObject =
    crate::lisp::LispObject(1024 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qreleased_button: LispObject =
    crate::lisp::LispObject(1025 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qremap: LispObject =
    crate::lisp::LispObject(1026 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qremote_file_error: LispObject =
    crate::lisp::LispObject(1027 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qrename_file: LispObject =
    crate::lisp::LispObject(1028 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qreplace_buffer_in_windows: LispObject =
    crate::lisp::LispObject(1029 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qrequire: LispObject =
    crate::lisp::LispObject(1030 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qreturn: LispObject =
    crate::lisp::LispObject(1031 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qright: LispObject =
    crate::lisp::LispObject(1032 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qright_divider: LispObject =
    crate::lisp::LispObject(1033 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qright_divider_width: LispObject =
    crate::lisp::LispObject(1034 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qright_edge: LispObject =
    crate::lisp::LispObject(1035 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qright_fringe: LispObject =
    crate::lisp::LispObject(1036 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qright_margin: LispObject =
    crate::lisp::LispObject(1037 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qright_to_left: LispObject =
    crate::lisp::LispObject(1038 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qrightmost: LispObject =
    crate::lisp::LispObject(1039 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qrisky_local_variable: LispObject =
    crate::lisp::LispObject(1040 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qrotate: LispObject =
    crate::lisp::LispObject(1041 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qrotate90: LispObject =
    crate::lisp::LispObject(1042 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qrss: LispObject =
    crate::lisp::LispObject(1043 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qrun: LispObject =
    crate::lisp::LispObject(1044 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qrun_hook_query_error_with_timeout: LispObject =
    crate::lisp::LispObject(1045 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qrun_hook_with_args: LispObject =
    crate::lisp::LispObject(1046 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qrun_with_timer: LispObject =
    crate::lisp::LispObject(1047 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qsafe: LispObject =
    crate::lisp::LispObject(1048 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qsave_excursion: LispObject =
    crate::lisp::LispObject(1049 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qsave_session: LispObject =
    crate::lisp::LispObject(1050 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qscale: LispObject =
    crate::lisp::LispObject(1051 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qscan_error: LispObject =
    crate::lisp::LispObject(1052 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qscratch: LispObject =
    crate::lisp::LispObject(1053 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qscreen_gamma: LispObject =
    crate::lisp::LispObject(1054 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qscroll_bar: LispObject =
    crate::lisp::LispObject(1055 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qscroll_bar_background: LispObject =
    crate::lisp::LispObject(1056 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qscroll_bar_foreground: LispObject =
    crate::lisp::LispObject(1057 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qscroll_bar_height: LispObject =
    crate::lisp::LispObject(1058 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qscroll_bar_movement: LispObject =
    crate::lisp::LispObject(1059 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qscroll_bar_width: LispObject =
    crate::lisp::LispObject(1060 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qscroll_command: LispObject =
    crate::lisp::LispObject(1061 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qscroll_down: LispObject =
    crate::lisp::LispObject(1062 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qscroll_up: LispObject =
    crate::lisp::LispObject(1063 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qsearch_failed: LispObject =
    crate::lisp::LispObject(1064 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qselect_window: LispObject =
    crate::lisp::LispObject(1065 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qselection_request: LispObject =
    crate::lisp::LispObject(1066 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qsemi_bold: LispObject =
    crate::lisp::LispObject(1067 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qsemi_light: LispObject =
    crate::lisp::LispObject(1068 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qseqpacket: LispObject =
    crate::lisp::LispObject(1069 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qsequencep: LispObject =
    crate::lisp::LispObject(1070 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qserial: LispObject =
    crate::lisp::LispObject(1071 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qsess: LispObject =
    crate::lisp::LispObject(1072 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qset: LispObject =
    crate::lisp::LispObject(1073 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qset_args_to_local: LispObject =
    crate::lisp::LispObject(1074 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qset_default: LispObject =
    crate::lisp::LispObject(1075 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qset_file_acl: LispObject =
    crate::lisp::LispObject(1076 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qset_file_modes: LispObject =
    crate::lisp::LispObject(1077 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qset_file_selinux_context: LispObject =
    crate::lisp::LispObject(1078 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qset_file_times: LispObject =
    crate::lisp::LispObject(1079 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qset_frame_size: LispObject =
    crate::lisp::LispObject(1080 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qset_internal: LispObject =
    crate::lisp::LispObject(1081 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qset_par_to_local: LispObject =
    crate::lisp::LispObject(1082 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qset_rest_args_to_local: LispObject =
    crate::lisp::LispObject(1083 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qset_visited_file_modtime: LispObject =
    crate::lisp::LispObject(1084 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qset_window_configuration: LispObject =
    crate::lisp::LispObject(1085 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qsetcar: LispObject =
    crate::lisp::LispObject(1086 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qsetcdr: LispObject =
    crate::lisp::LispObject(1087 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qsetimm: LispObject =
    crate::lisp::LispObject(1088 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qsetq: LispObject =
    crate::lisp::LispObject(1089 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qsetting_constant: LispObject =
    crate::lisp::LispObject(1090 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qsha1: LispObject =
    crate::lisp::LispObject(1091 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qsha224: LispObject =
    crate::lisp::LispObject(1092 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qsha256: LispObject =
    crate::lisp::LispObject(1093 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qsha384: LispObject =
    crate::lisp::LispObject(1094 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qsha512: LispObject =
    crate::lisp::LispObject(1095 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qshift_jis: LispObject =
    crate::lisp::LispObject(1096 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qshlwapi: LispObject =
    crate::lisp::LispObject(1097 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qside_effect_free: LispObject =
    crate::lisp::LispObject(1098 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qsignal: LispObject =
    crate::lisp::LispObject(1099 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qsingularity_error: LispObject =
    crate::lisp::LispObject(1100 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qsize: LispObject =
    crate::lisp::LispObject(1101 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qskip_taskbar: LispObject =
    crate::lisp::LispObject(1102 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qslice: LispObject =
    crate::lisp::LispObject(1103 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qsound: LispObject =
    crate::lisp::LispObject(1104 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qsource: LispObject =
    crate::lisp::LispObject(1105 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qspace: LispObject =
    crate::lisp::LispObject(1106 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qspace_width: LispObject =
    crate::lisp::LispObject(1107 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qspecial_lowercase: LispObject =
    crate::lisp::LispObject(1108 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qspecial_titlecase: LispObject =
    crate::lisp::LispObject(1109 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qspecial_uppercase: LispObject =
    crate::lisp::LispObject(1110 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qstandard_input: LispObject =
    crate::lisp::LispObject(1111 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qstandard_output: LispObject =
    crate::lisp::LispObject(1112 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qstart: LispObject =
    crate::lisp::LispObject(1113 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qstart_process: LispObject =
    crate::lisp::LispObject(1114 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qstate: LispObject =
    crate::lisp::LispObject(1115 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qstderr: LispObject =
    crate::lisp::LispObject(1116 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qstdin: LispObject =
    crate::lisp::LispObject(1117 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qstdout: LispObject =
    crate::lisp::LispObject(1118 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qsticky: LispObject =
    crate::lisp::LispObject(1119 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qstime: LispObject =
    crate::lisp::LispObject(1120 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qstop: LispObject =
    crate::lisp::LispObject(1121 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qstraight: LispObject =
    crate::lisp::LispObject(1122 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qstring: LispObject =
    crate::lisp::LispObject(1123 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qstring_bytes: LispObject =
    crate::lisp::LispObject(1124 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qstring_lessp: LispObject =
    crate::lisp::LispObject(1125 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qstring_without_embedded_nulls_p: LispObject =
    crate::lisp::LispObject(1126 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qstringp: LispObject =
    crate::lisp::LispObject(1127 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qstrings: LispObject =
    crate::lisp::LispObject(1128 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qsub1: LispObject =
    crate::lisp::LispObject(1129 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qsubfeatures: LispObject =
    crate::lisp::LispObject(1130 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qsubr: LispObject =
    crate::lisp::LispObject(1131 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qsubrp: LispObject =
    crate::lisp::LispObject(1132 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qsubstitute_command_keys: LispObject =
    crate::lisp::LispObject(1133 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qsubstitute_env_in_file_name: LispObject =
    crate::lisp::LispObject(1134 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qsubstitute_in_file_name: LispObject =
    crate::lisp::LispObject(1135 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qsvg: LispObject =
    crate::lisp::LispObject(1136 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qsw: LispObject =
    crate::lisp::LispObject(1137 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qswitch_frame: LispObject =
    crate::lisp::LispObject(1138 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qsymbol: LispObject =
    crate::lisp::LispObject(1139 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qsymbolp: LispObject =
    crate::lisp::LispObject(1140 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qsymbols: LispObject =
    crate::lisp::LispObject(1141 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qsyntax_ppss: LispObject =
    crate::lisp::LispObject(1142 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qsyntax_ppss_flush_cache: LispObject =
    crate::lisp::LispObject(1143 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qsyntax_table: LispObject =
    crate::lisp::LispObject(1144 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qsyntax_table_p: LispObject =
    crate::lisp::LispObject(1145 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtab_bar: LispObject =
    crate::lisp::LispObject(1146 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtab_bar_lines: LispObject =
    crate::lisp::LispObject(1147 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtab_bar_size: LispObject =
    crate::lisp::LispObject(1148 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtab_line: LispObject =
    crate::lisp::LispObject(1149 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtab_line_format: LispObject =
    crate::lisp::LispObject(1150 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtarget_idx: LispObject =
    crate::lisp::LispObject(1151 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtb_size_cb: LispObject =
    crate::lisp::LispObject(1152 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtemp_buffer_setup_hook: LispObject =
    crate::lisp::LispObject(1153 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtemp_buffer_show_hook: LispObject =
    crate::lisp::LispObject(1154 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qterminal: LispObject =
    crate::lisp::LispObject(1155 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qterminal_frame: LispObject =
    crate::lisp::LispObject(1156 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qterminal_live_p: LispObject =
    crate::lisp::LispObject(1157 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtest: LispObject =
    crate::lisp::LispObject(1158 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtext: LispObject =
    crate::lisp::LispObject(1159 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtext_image_horiz: LispObject =
    crate::lisp::LispObject(1160 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtext_pixels: LispObject =
    crate::lisp::LispObject(1161 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtext_read_only: LispObject =
    crate::lisp::LispObject(1162 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qthcount: LispObject =
    crate::lisp::LispObject(1163 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qthin_space: LispObject =
    crate::lisp::LispObject(1164 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qthread: LispObject =
    crate::lisp::LispObject(1165 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qthread_event: LispObject =
    crate::lisp::LispObject(1166 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qthreadp: LispObject =
    crate::lisp::LispObject(1167 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtiff: LispObject =
    crate::lisp::LispObject(1168 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtime: LispObject =
    crate::lisp::LispObject(1169 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtimer_event_handler: LispObject =
    crate::lisp::LispObject(1170 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtitle: LispObject =
    crate::lisp::LispObject(1171 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtitle_bar_size: LispObject =
    crate::lisp::LispObject(1172 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtitlecase: LispObject =
    crate::lisp::LispObject(1173 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtool_bar: LispObject =
    crate::lisp::LispObject(1174 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtool_bar_external: LispObject =
    crate::lisp::LispObject(1175 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtool_bar_lines: LispObject =
    crate::lisp::LispObject(1176 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtool_bar_position: LispObject =
    crate::lisp::LispObject(1177 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtool_bar_size: LispObject =
    crate::lisp::LispObject(1178 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtooltip: LispObject =
    crate::lisp::LispObject(1179 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtop: LispObject =
    crate::lisp::LispObject(1180 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtop_bottom: LispObject =
    crate::lisp::LispObject(1181 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtop_edge: LispObject =
    crate::lisp::LispObject(1182 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtop_left_corner: LispObject =
    crate::lisp::LispObject(1183 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtop_level: LispObject =
    crate::lisp::LispObject(1184 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtop_only: LispObject =
    crate::lisp::LispObject(1185 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtop_right_corner: LispObject =
    crate::lisp::LispObject(1186 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtpgid: LispObject =
    crate::lisp::LispObject(1187 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtrailing_whitespace: LispObject =
    crate::lisp::LispObject(1188 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtranslation_table: LispObject =
    crate::lisp::LispObject(1189 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtranslation_table_id: LispObject =
    crate::lisp::LispObject(1190 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtrapping_constant: LispObject =
    crate::lisp::LispObject(1191 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtruncation: LispObject =
    crate::lisp::LispObject(1192 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qttname: LispObject =
    crate::lisp::LispObject(1193 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtty: LispObject =
    crate::lisp::LispObject(1194 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtty_color_alist: LispObject =
    crate::lisp::LispObject(1195 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtty_color_by_index: LispObject =
    crate::lisp::LispObject(1196 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtty_color_desc: LispObject =
    crate::lisp::LispObject(1197 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtty_color_mode: LispObject =
    crate::lisp::LispObject(1198 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtty_color_standard_values: LispObject =
    crate::lisp::LispObject(1199 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtty_menu_exit: LispObject =
    crate::lisp::LispObject(1200 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtty_menu_ignore: LispObject =
    crate::lisp::LispObject(1201 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtty_menu_mouse_movement: LispObject =
    crate::lisp::LispObject(1202 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtty_menu_navigation_map: LispObject =
    crate::lisp::LispObject(1203 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtty_menu_next_item: LispObject =
    crate::lisp::LispObject(1204 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtty_menu_next_menu: LispObject =
    crate::lisp::LispObject(1205 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtty_menu_prev_item: LispObject =
    crate::lisp::LispObject(1206 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtty_menu_prev_menu: LispObject =
    crate::lisp::LispObject(1207 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtty_menu_select: LispObject =
    crate::lisp::LispObject(1208 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtty_mode_reset_strings: LispObject =
    crate::lisp::LispObject(1209 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtty_mode_set_strings: LispObject =
    crate::lisp::LispObject(1210 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qtty_type: LispObject =
    crate::lisp::LispObject(1211 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qultra_bold: LispObject =
    crate::lisp::LispObject(1212 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qundecided: LispObject =
    crate::lisp::LispObject(1213 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qundecorated: LispObject =
    crate::lisp::LispObject(1214 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qundefined: LispObject =
    crate::lisp::LispObject(1215 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qunderflow_error: LispObject =
    crate::lisp::LispObject(1216 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qunderline_minimum_offset: LispObject =
    crate::lisp::LispObject(1217 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qundo_auto__add_boundary: LispObject =
    crate::lisp::LispObject(1218 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qundo_auto__last_boundary_cause: LispObject =
    crate::lisp::LispObject(1219 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qundo_auto__this_command_amalgamating: LispObject =
    crate::lisp::LispObject(1220 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qundo_auto__undoable_change: LispObject =
    crate::lisp::LispObject(1221 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qundo_auto__undoably_changed_buffers: LispObject =
    crate::lisp::LispObject(1222 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qundo_auto_amalgamate: LispObject =
    crate::lisp::LispObject(1223 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qunevalled: LispObject =
    crate::lisp::LispObject(1224 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qunhandled_file_name_directory: LispObject =
    crate::lisp::LispObject(1225 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qunicode: LispObject =
    crate::lisp::LispObject(1226 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qunicode_bmp: LispObject =
    crate::lisp::LispObject(1227 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qunicode_string_p: LispObject =
    crate::lisp::LispObject(1228 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qunix: LispObject =
    crate::lisp::LispObject(1229 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qunlet: LispObject =
    crate::lisp::LispObject(1230 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qunmount: LispObject =
    crate::lisp::LispObject(1231 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qunreachable: LispObject =
    crate::lisp::LispObject(1232 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qunspecified: LispObject =
    crate::lisp::LispObject(1233 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qunsplittable: LispObject =
    crate::lisp::LispObject(1234 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qup: LispObject =
    crate::lisp::LispObject(1235 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qupdate_frame_menubar: LispObject =
    crate::lisp::LispObject(1236 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qupdate_frame_tab_bar: LispObject =
    crate::lisp::LispObject(1237 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qupdate_frame_tool_bar: LispObject =
    crate::lisp::LispObject(1238 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qus_ascii: LispObject =
    crate::lisp::LispObject(1239 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Quser: LispObject =
    crate::lisp::LispObject(1240 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Quser_error: LispObject =
    crate::lisp::LispObject(1241 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Quser_position: LispObject =
    crate::lisp::LispObject(1242 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Quser_ptr: LispObject =
    crate::lisp::LispObject(1243 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Quser_ptrp: LispObject =
    crate::lisp::LispObject(1244 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Quser_search_failed: LispObject =
    crate::lisp::LispObject(1245 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Quser_size: LispObject =
    crate::lisp::LispObject(1246 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qutf_16: LispObject =
    crate::lisp::LispObject(1247 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qutf_16le: LispObject =
    crate::lisp::LispObject(1248 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qutf_8: LispObject =
    crate::lisp::LispObject(1249 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qutf_8_emacs: LispObject =
    crate::lisp::LispObject(1250 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qutf_8_string_p: LispObject =
    crate::lisp::LispObject(1251 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qutf_8_unix: LispObject =
    crate::lisp::LispObject(1252 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qutime: LispObject =
    crate::lisp::LispObject(1253 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qvalue: LispObject =
    crate::lisp::LispObject(1254 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qvariable_documentation: LispObject =
    crate::lisp::LispObject(1255 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qvector: LispObject =
    crate::lisp::LispObject(1256 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qvector_or_char_table_p: LispObject =
    crate::lisp::LispObject(1257 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qvector_slots: LispObject =
    crate::lisp::LispObject(1258 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qvectorp: LispObject =
    crate::lisp::LispObject(1259 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qvectors: LispObject =
    crate::lisp::LispObject(1260 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qverify_visited_file_modtime: LispObject =
    crate::lisp::LispObject(1261 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qvertical_border: LispObject =
    crate::lisp::LispObject(1262 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qvertical_line: LispObject =
    crate::lisp::LispObject(1263 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qvertical_scroll_bar: LispObject =
    crate::lisp::LispObject(1264 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qvertical_scroll_bars: LispObject =
    crate::lisp::LispObject(1265 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qvisibility: LispObject =
    crate::lisp::LispObject(1266 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qvisible: LispObject =
    crate::lisp::LispObject(1267 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qvisual: LispObject =
    crate::lisp::LispObject(1268 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qvoid_function: LispObject =
    crate::lisp::LispObject(1269 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qvoid_variable: LispObject =
    crate::lisp::LispObject(1270 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qvsize: LispObject =
    crate::lisp::LispObject(1271 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qw32: LispObject =
    crate::lisp::LispObject(1272 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qwait_for_wm: LispObject =
    crate::lisp::LispObject(1273 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qwall: LispObject =
    crate::lisp::LispObject(1274 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qwatchers: LispObject =
    crate::lisp::LispObject(1275 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qwave: LispObject =
    crate::lisp::LispObject(1276 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qweakness: LispObject =
    crate::lisp::LispObject(1277 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qwhen: LispObject =
    crate::lisp::LispObject(1278 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qwholenump: LispObject =
    crate::lisp::LispObject(1279 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qwidget_type: LispObject =
    crate::lisp::LispObject(1280 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qwidth: LispObject =
    crate::lisp::LispObject(1281 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qwidth_only: LispObject =
    crate::lisp::LispObject(1282 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qwindow: LispObject =
    crate::lisp::LispObject(1283 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qwindow__pixel_to_total: LispObject =
    crate::lisp::LispObject(1284 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qwindow__resize_mini_frame: LispObject =
    crate::lisp::LispObject(1285 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qwindow__resize_root_window: LispObject =
    crate::lisp::LispObject(1286 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qwindow__resize_root_window_vertically: LispObject =
    crate::lisp::LispObject(1287 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qwindow__sanitize_window_sizes: LispObject =
    crate::lisp::LispObject(1288 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qwindow_buffer_change_functions: LispObject =
    crate::lisp::LispObject(1289 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qwindow_configuration: LispObject =
    crate::lisp::LispObject(1290 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qwindow_configuration_change_hook: LispObject =
    crate::lisp::LispObject(1291 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qwindow_configuration_p: LispObject =
    crate::lisp::LispObject(1292 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qwindow_deletable_p: LispObject =
    crate::lisp::LispObject(1293 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qwindow_divider: LispObject =
    crate::lisp::LispObject(1294 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qwindow_divider_first_pixel: LispObject =
    crate::lisp::LispObject(1295 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qwindow_divider_last_pixel: LispObject =
    crate::lisp::LispObject(1296 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qwindow_id: LispObject =
    crate::lisp::LispObject(1297 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qwindow_live_p: LispObject =
    crate::lisp::LispObject(1298 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qwindow_point_insertion_type: LispObject =
    crate::lisp::LispObject(1299 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qwindow_scroll_functions: LispObject =
    crate::lisp::LispObject(1300 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qwindow_selection_change_functions: LispObject =
    crate::lisp::LispObject(1301 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qwindow_size: LispObject =
    crate::lisp::LispObject(1302 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qwindow_size_change_functions: LispObject =
    crate::lisp::LispObject(1303 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qwindow_state_change_functions: LispObject =
    crate::lisp::LispObject(1304 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qwindow_state_change_hook: LispObject =
    crate::lisp::LispObject(1305 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qwindow_valid_p: LispObject =
    crate::lisp::LispObject(1306 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qwindowp: LispObject =
    crate::lisp::LispObject(1307 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qworkarea: LispObject =
    crate::lisp::LispObject(1308 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qwr: LispObject =
    crate::lisp::LispObject(1309 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qwrap_prefix: LispObject =
    crate::lisp::LispObject(1310 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qwrite_region: LispObject =
    crate::lisp::LispObject(1311 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qwrite_region_annotate_functions: LispObject =
    crate::lisp::LispObject(1312 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qwrong_length_argument: LispObject =
    crate::lisp::LispObject(1313 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qwrong_number_of_arguments: LispObject =
    crate::lisp::LispObject(1314 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qwrong_register_subr_call: LispObject =
    crate::lisp::LispObject(1315 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qwrong_type_argument: LispObject =
    crate::lisp::LispObject(1316 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qx: LispObject =
    crate::lisp::LispObject(1317 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qx_check_fullscreen: LispObject =
    crate::lisp::LispObject(1318 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qx_create_frame_1: LispObject =
    crate::lisp::LispObject(1319 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qx_create_frame_2: LispObject =
    crate::lisp::LispObject(1320 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qx_frame_parameter: LispObject =
    crate::lisp::LispObject(1321 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qx_handle_net_wm_state: LispObject =
    crate::lisp::LispObject(1322 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qx_net_wm_state: LispObject =
    crate::lisp::LispObject(1323 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qx_resource_name: LispObject =
    crate::lisp::LispObject(1324 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qx_set_frame_parameters: LispObject =
    crate::lisp::LispObject(1325 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qx_set_fullscreen: LispObject =
    crate::lisp::LispObject(1326 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qx_set_menu_bar_lines: LispObject =
    crate::lisp::LispObject(1327 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qx_set_window_size_1: LispObject =
    crate::lisp::LispObject(1328 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qx_set_window_size_2: LispObject =
    crate::lisp::LispObject(1329 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qx_set_window_size_3: LispObject =
    crate::lisp::LispObject(1330 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qxbm: LispObject =
    crate::lisp::LispObject(1331 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qxg_change_toolbar_position: LispObject =
    crate::lisp::LispObject(1332 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qxg_frame_resized: LispObject =
    crate::lisp::LispObject(1333 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qxg_frame_set_char_size: LispObject =
    crate::lisp::LispObject(1334 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qxg_frame_set_char_size_1: LispObject =
    crate::lisp::LispObject(1335 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qxg_frame_set_char_size_2: LispObject =
    crate::lisp::LispObject(1336 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qxg_frame_set_char_size_3: LispObject =
    crate::lisp::LispObject(1337 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qxg_frame_set_char_size_4: LispObject =
    crate::lisp::LispObject(1338 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qxpm: LispObject =
    crate::lisp::LispObject(1339 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qxwidget: LispObject =
    crate::lisp::LispObject(1340 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qxwidget_event: LispObject =
    crate::lisp::LispObject(1341 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qxwidget_view: LispObject =
    crate::lisp::LispObject(1342 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qyes_or_no_p_history: LispObject =
    crate::lisp::LispObject(1343 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qz_group: LispObject =
    crate::lisp::LispObject(1344 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
pub const Qzero_width: LispObject =
    crate::lisp::LispObject(1345 * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));
