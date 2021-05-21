use deno::create_main_worker;
use deno::file_fetcher::File;
use deno::file_watcher;
use deno::file_watcher::ModuleResolutionResult;
use deno::flags::Flags;
use deno::media_type::MediaType;
use deno::program_state::ProgramState;
use deno::specifier_handler::FetchHandler;
use deno::{fs_util, module_graph, tools};
use deno_core::error::AnyError;
use deno_core::resolve_url_or_path;
use deno_core::ModuleSpecifier;
use deno_runtime::permissions::Permissions;

use crate::futures::FutureExt;

use std::env;
use std::io::Read;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

// Forked from https://github.com/denoland/deno/
// The context of this file is that we wanted to add lisp interfacing with
// these subcommands. In order to do that, we just need to call
// v8_bind_lisp_funcs on the workers. When upgrading deno, you will
// likely need to refork these functions.
// Copyright 2018-2021 the Deno authors. All rights reserved. MIT license.
pub(crate) async fn run_repl(flags: Flags) -> Result<(), AnyError> {
    let main_module = resolve_url_or_path("./$deno$repl.ts").unwrap();
    let permissions = Permissions::from_options(&flags.clone().into());
    let program_state = ProgramState::build(flags).await?;
    let mut worker = create_main_worker(&program_state, main_module.clone(), permissions);
    crate::javascript::v8_bind_lisp_funcs(&mut worker)?;
    worker.run_event_loop().await?;

    tools::repl::run(&program_state, worker).await
}

pub(crate) async fn eval_command(
    flags: Flags,
    code: String,
    ext: String,
    print: bool,
) -> Result<(), AnyError> {
    // Force TypeScript compile.
    let main_module = resolve_url_or_path("./$deno$eval.ts").unwrap();
    let permissions = Permissions::from_options(&flags.clone().into());
    let program_state = ProgramState::build(flags).await?;
    let mut worker = create_main_worker(&program_state, main_module.clone(), permissions);
    crate::javascript::v8_bind_lisp_funcs(&mut worker)?;
    // Create a dummy source file.
    let source_code = if print {
        format!("console.log({})", code)
    } else {
        code
    }
    .into_bytes();

    let file = File {
        local: main_module.clone().to_file_path().unwrap(),
        maybe_types: None,
        media_type: if ext.as_str() == "ts" {
            MediaType::TypeScript
        } else if ext.as_str() == "tsx" {
            MediaType::Tsx
        } else if ext.as_str() == "js" {
            MediaType::JavaScript
        } else {
            MediaType::Jsx
        },
        source: String::from_utf8(source_code)?,
        specifier: main_module.clone(),
    };

    // Save our fake file into file fetcher cache
    // to allow module access by TS compiler.
    program_state.file_fetcher.insert_cached(file);
    //  debug!("main_module {}", &main_module);
    worker.execute_module(&main_module).await?;
    worker.execute("window.dispatchEvent(new Event('load'))")?;
    worker.run_event_loop().await?;
    worker.execute("window.dispatchEvent(new Event('unload'))")?;
    Ok(())
}

async fn run_from_stdin(flags: Flags) -> Result<(), AnyError> {
    let program_state = ProgramState::build(flags.clone()).await?;
    let permissions = Permissions::from_options(&flags.clone().into());
    let main_module = resolve_url_or_path("./$deno$stdin.ts").unwrap();
    let mut worker = create_main_worker(&program_state.clone(), main_module.clone(), permissions);

    crate::javascript::v8_bind_lisp_funcs(&mut worker)?;
    let mut source = Vec::new();
    std::io::stdin().read_to_end(&mut source)?;
    // Create a dummy source file.
    let source_file = File {
        local: main_module.clone().to_file_path().unwrap(),
        maybe_types: None,
        media_type: MediaType::TypeScript,
        source: String::from_utf8(source)?,
        specifier: main_module.clone(),
    };
    // Save our fake file into file fetcher cache
    // to allow module access by TS compiler
    program_state.file_fetcher.insert_cached(source_file);

    //  debug!("main_module {}", main_module);
    worker.execute_module(&main_module).await?;
    worker.execute("window.dispatchEvent(new Event('load'))")?;
    worker.run_event_loop().await?;
    worker.execute("window.dispatchEvent(new Event('unload'))")?;
    Ok(())
}

async fn run_with_watch(flags: Flags, script: String) -> Result<(), AnyError> {
    let module_resolver = || {
        let script1 = script.clone();
        let script2 = script.clone();
        let flags = flags.clone();
        async move {
            let main_module = resolve_url_or_path(&script1)?;
            let program_state = ProgramState::build(flags).await?;
            let handler = Arc::new(Mutex::new(FetchHandler::new(
                &program_state,
                Permissions::allow_all(),
            )?));
            let mut builder = module_graph::GraphBuilder::new(
                handler,
                program_state.maybe_import_map.clone(),
                program_state.lockfile.clone(),
            );
            builder.add(&main_module, false).await?;
            let module_graph = builder.get_graph();

            // Find all local files in graph
            let mut paths_to_watch: Vec<PathBuf> = module_graph
                .get_modules()
                .iter()
                .filter_map(|specifier| specifier.to_file_path().ok())
                .collect();

            if let Some(import_map) = program_state.flags.import_map_path.as_ref() {
                paths_to_watch.push(fs_util::resolve_from_cwd(std::path::Path::new(import_map))?);
            }

            Ok((paths_to_watch, main_module))
        }
        .map(move |result| match result {
            Ok((paths_to_watch, module_info)) => ModuleResolutionResult::Success {
                paths_to_watch,
                module_info,
            },
            Err(e) => ModuleResolutionResult::Fail {
                source_path: PathBuf::from(script2),
                error: e,
            },
        })
        .boxed_local()
    };

    let operation = |main_module: ModuleSpecifier| {
        let flags = flags.clone();
        let permissions = Permissions::from_options(&flags.clone().into());
        async move {
            let main_module = main_module.clone();
            let program_state = ProgramState::build(flags).await?;
            let mut worker = create_main_worker(&program_state, main_module.clone(), permissions);
            crate::javascript::v8_bind_lisp_funcs(&mut worker)?;
            //      debug!("main_module {}", main_module);
            worker.execute_module(&main_module).await?;
            worker.execute("window.dispatchEvent(new Event('load'))")?;
            worker.run_event_loop().await?;
            worker.execute("window.dispatchEvent(new Event('unload'))")?;
            Ok(())
        }
        .boxed_local()
    };

    file_watcher::watch_func_with_module_resolution(module_resolver, operation, "Process").await
}

pub(crate) async fn run_command(flags: Flags, script: String) -> Result<(), AnyError> {
    // Read script content from stdin
    if script == "-" {
        return run_from_stdin(flags).await;
    }

    if flags.watch {
        return run_with_watch(flags, script).await;
    }

    let main_module = resolve_url_or_path(&script)?;
    let program_state = ProgramState::build(flags.clone()).await?;
    let permissions = Permissions::from_options(&flags.clone().into());
    let mut worker = create_main_worker(&program_state, main_module.clone(), permissions);
    crate::javascript::v8_bind_lisp_funcs(&mut worker)?;

    let mut maybe_coverage_collector = if let Some(ref coverage_dir) = program_state.coverage_dir {
        let session = worker.create_inspector_session();

        let coverage_dir = PathBuf::from(coverage_dir);
        let mut coverage_collector = tools::coverage::CoverageCollector::new(coverage_dir, session);
        coverage_collector.start_collecting().await?;

        Some(coverage_collector)
    } else {
        None
    };

    //  debug!("main_module {}", main_module);
    worker.execute_module(&main_module).await?;
    worker.execute("window.dispatchEvent(new Event('load'))")?;
    worker.run_event_loop().await?;
    worker.execute("window.dispatchEvent(new Event('unload'))")?;

    if let Some(coverage_collector) = maybe_coverage_collector.as_mut() {
        coverage_collector.stop_collecting().await?;
    }

    Ok(())
}

pub(crate) async fn test_command(
    flags: Flags,
    include: Option<Vec<String>>,
    no_run: bool,
    fail_fast: bool,
    quiet: bool,
    allow_none: bool,
    filter: Option<String>,
) -> Result<(), AnyError> {
    let program_state = ProgramState::build(flags.clone()).await?;
    let permissions = Permissions::from_options(&flags.clone().into());
    let cwd = std::env::current_dir().expect("No current directory");
    let include = include.unwrap_or_else(|| vec![".".to_string()]);
    let test_modules = tools::test_runner::prepare_test_modules_urls(include, &cwd)?;

    if test_modules.is_empty() {
        println!("No matching test modules found");
        if !allow_none {
            std::process::exit(1);
        }
        return Ok(());
    }
    let main_module = deno_core::resolve_path("$deno$test.ts")?;
    // Create a dummy source file.
    let source_file = File {
        local: main_module.to_file_path().unwrap(),
        maybe_types: None,
        media_type: MediaType::TypeScript,
        source: tools::test_runner::render_test_file(
            test_modules.clone(),
            fail_fast,
            quiet,
            filter,
        ),
        specifier: main_module.clone(),
    };
    // Save our fake file into file fetcher cache
    // to allow module access by TS compiler
    program_state.file_fetcher.insert_cached(source_file);

    if no_run {
        let lib = if flags.unstable {
            module_graph::TypeLib::UnstableDenoWindow
        } else {
            module_graph::TypeLib::DenoWindow
        };
        program_state
            .prepare_module_load(
                main_module.clone(),
                lib,
                Permissions::allow_all(),
                false,
                program_state.maybe_import_map.clone(),
            )
            .await?;
        return Ok(());
    }

    let mut worker = create_main_worker(&program_state, main_module.clone(), permissions);
    crate::javascript::v8_bind_lisp_funcs(&mut worker)?;

    if let Some(ref coverage_dir) = flags.coverage_dir {
        env::set_var("DENO_UNSTABLE_COVERAGE_DIR", coverage_dir);
    }

    let mut maybe_coverage_collector = if let Some(ref coverage_dir) = program_state.coverage_dir {
        let session = worker.create_inspector_session();
        let coverage_dir = PathBuf::from(coverage_dir);
        let mut coverage_collector = tools::coverage::CoverageCollector::new(coverage_dir, session);
        coverage_collector.start_collecting().await?;

        Some(coverage_collector)
    } else {
        None
    };

    let execute_result = worker.execute_module(&main_module).await;
    execute_result?;
    worker.execute("window.dispatchEvent(new Event('load'))")?;
    worker.run_event_loop().await?;
    worker.execute("window.dispatchEvent(new Event('unload'))")?;
    worker.run_event_loop().await?;

    if let Some(coverage_collector) = maybe_coverage_collector.as_mut() {
        coverage_collector.stop_collecting().await?;
    }

    Ok(())
}
