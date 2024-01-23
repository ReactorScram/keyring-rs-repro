fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    tracing::info!("Single thread test");
    hammer_thread(0);

    tracing::info!("Multi-thread test");
    let handles = (0..2).map(|i| std::thread::spawn(move || {
        hammer_thread(i)
    })).collect::<Vec<_>>();

    for handle in handles {
        handle.join().unwrap();
    }

    Ok(())
}

fn hammer_thread(i: u32) {
    let name = format!("dev.firezone.client/test_OZQP3QIN/token/{i}");

    for _ in 0..1000 {
        if let Err(error) = hammer_cycle(&name) {
            tracing::error!(?error, "Multi-thread test failed");
        }
    }

    keyring::Entry::new_with_target(&name, "", "").unwrap().delete_password().ok();
}

fn hammer_cycle(name: &str) -> anyhow::Result<()> {
    let password = "bogus_password";

    keyring::Entry::new_with_target(name, "", "")?.delete_password().ok();
    keyring::Entry::new_with_target(name, "", "")?.set_password(password)?;

    for iteration in 0..5 {
        let Ok(actual) = keyring::Entry::new_with_target(name, "", "")?
            .get_password()
        else {
            tracing::error!(?name, ?iteration, "couldn't get_password");
            std::thread::sleep(std::time::Duration::from_millis(1));
            continue;
        };
        if actual != password {
           anyhow::bail!("retrieved password didn't match stored password");
        }
        break;
    }

    Ok(())
}
