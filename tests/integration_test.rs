mod mocks;

use mkchlog::changelog::Changelog;
use mkchlog::git::Git;
use mkchlog::template::Template;
use mocks::GitCmdMock;
use std::fs::File;

const YAML_FILE: &str = ".mkchlog.yml";

#[test]
fn it_produces_correct_output() {
    let mocked_log = String::from(
        "\
commit 1cc72956df91e2fd8c45e72983c4e1149f1ac3b3
Author: Cry Wolf <cry.wolf@centrum.cz>
Date:   Tue Jun 13 16:27:59 2023 +0200

    Fixed TOCTOU race condition when opening file

    Previously we checked the file permissions before opening
    the file now we check the metadata using file descriptor
    after opening the file. (before reading)

    changelog:
        section: security:vuln_fixes
        title: Fixed vulnerability related to opening files
        description: The application was vulnerable to attacks
                     if the attacker had access to the working
                     directory. If you run this in such
                     enviroment you should update ASAP. If your
                     working directory is **not** accessible by
                     unprivileged users you don't need to worry.

commit 7c85bee4303d56bededdfacf8fbb7bdc68e2195b
Author: Cry Wolf <cry.wolf@centrum.cz>
Date:   Tue Jun 13 16:26:35 2023 +0200

    Don't reallocate the buffer when we know its size

    This computes the size and allocates the buffer upfront.
    Avoiding allocations like this introduces 10% speedup.

    changelog:
        section: perf
        title: Improved processing speed by 10%
        title-is-enough: true

commit a1a654e256cc96e1c4b5a81845b5e3f3f0aa9ed3
Author: Cry Wolf <cry.wolf@centrum.cz>
Date:   Tue Jun 13 16:25:29 2023 +0200

    Fixed grammar mistakes.

    We found 42 grammar mistakes that are fixed in this commit.

    changelog: skip

commit 62db026b0ead7f0659df10c70e402c70ede5d7dd
Author: Cry Wolf <cry.wolf@centrum.cz>
Date:   Tue Jun 13 16:24:22 2023 +0200

    Added ability to skip commits.

    This allows commits to be skipped by typing 'changelog: skip'
    at the end of the commit. This is mainly useful for typo
    fixes or other things irrelevant to the user of a project.

    changelog:
        inherit: all
        section: features",
    );

    let git_cmd = Box::new(GitCmdMock::new(mocked_log));
    let git = Git::new(git_cmd);

    let f = File::open(YAML_FILE).unwrap();
    let template = Template::new(f).unwrap();
    let changelog = Changelog::new(template, git);

    let output = changelog.produce().unwrap();

    let exp_output = "\
============================================
## Security

This section contains very important security-related changes.

### Fixed vulnerabilities

#### Fixed vulnerability related to opening files

The application was vulnerable to attacks if the attacker had access to the working directory. \
If you run this in such enviroment you should update ASAP. \
If your working directory is **not** accessible by unprivileged users you don't need to worry.

## New features

### Added ability to skip commits.

This allows commits to be skipped by typing 'changelog: skip' \
at the end of the commit. This is mainly useful for typo fixes or other things irrelevant to the user of a project.

## Performance improvements

* Improved processing speed by 10%
============================================";

    assert_eq!(exp_output, output);
}

#[test]
fn only_sections_with_commits_shoud_be_printed_out() {
    let mocked_log = String::from(
        "\
commit 1cc72956df91e2fd8c45e72983c4e1149f1ac3b3
Author: Cry Wolf <cry.wolf@centrum.cz>
Date:   Tue Jun 13 16:27:59 2023 +0200

    Fixed TOCTOU race condition when opening file

    Previously we checked the file permissions before opening
    the file now we check the metadata using file descriptor
    after opening the file. (before reading)

    changelog:
        section: security:vuln_fixes
        title: Fixed vulnerability related to opening files
        description: The application was vulnerable to attacks
                     if the attacker had access to the working
                     directory. If you run this in such
                     enviroment you should update ASAP. If your
                     working directory is **not** accessible by
                     unprivileged users you don't need to worry.

",
    );

    let git_cmd = Box::new(GitCmdMock::new(mocked_log));
    let git = Git::new(git_cmd);

    let f = File::open(YAML_FILE).unwrap();
    let template = Template::new(f).unwrap();
    let changelog = Changelog::new(template, git);

    let output = changelog.produce().unwrap();

    let exp_output = "\
============================================
## Security

This section contains very important security-related changes.

### Fixed vulnerabilities

#### Fixed vulnerability related to opening files

The application was vulnerable to attacks if the attacker had access to the working directory. \
If you run this in such enviroment you should update ASAP. \
If your working directory is **not** accessible by unprivileged users you don't need to worry.
============================================";

    assert_eq!(exp_output, output);
}

#[test]
fn fails_when_unknown_section_in_commit() {
    let mocked_log = String::from(
        "\
commit 62db026b0ead7f0659df10c70e402c70ede5d7dd
Author: Cry Wolf <cry.wolf@centrum.cz>
Date:   Tue Jun 13 16:24:22 2023 +0200

    Added ability to skip commits.

    This allows commits to be skipped by typing 'changelog: skip'
    at the end of the commit. This is mainly useful for typo
    fixes or other things irrelevant to the user of a project.

    changelog:
        inherit: all
        section: unconfigured_section",
    );

    let git_cmd = Box::new(GitCmdMock::new(mocked_log));
    let git = Git::new(git_cmd);

    let f = File::open(YAML_FILE).unwrap();
    let template = Template::new(f).unwrap();
    let changelog = Changelog::new(template, git);

    let res = changelog.produce();

    assert!(res.is_err());
    assert!(res
        .unwrap_err()
        .to_string()
        .starts_with("Unknown section 'unconfigured_section' in changelog message"));
}

#[test]
fn fails_when_missing_section_key_in_commit() {
    let mocked_log = String::from(
        "\
commit 62db026b0ead7f0659df10c70e402c70ede5d7dd
Author: Cry Wolf <cry.wolf@centrum.cz>
Date:   Tue Jun 13 16:24:22 2023 +0200

    Added ability to skip commits.

    This allows commits to be skipped by typing 'changelog: skip'
    at the end of the commit. This is mainly useful for typo
    fixes or other things irrelevant to the user of a project.

    changelog:
        inherit: all",
    );

    let git_cmd = Box::new(GitCmdMock::new(mocked_log));
    let git = Git::new(git_cmd);

    let f = File::open(YAML_FILE).unwrap();
    let template = Template::new(f).unwrap();
    let changelog = Changelog::new(template, git);

    let res = changelog.produce();

    assert!(res.is_err());
    assert!(res
        .unwrap_err()
        .to_string()
        .starts_with("Missing 'section' key in changelog message:"));
}

#[test]
fn fails_when_only_one_line_in_commit() {
    let mocked_log = String::from(
        "\
commit 62db026b0ead7f0659df10c70e402c70ede5d7dd
Author: Cry Wolf <cry.wolf@centrum.cz>
Date:   Tue Jun 13 16:24:22 2023 +0200

    Added ability to skip commits.

    changelog:
        inherit: all
        section: features",
    );

    let git_cmd = Box::new(GitCmdMock::new(mocked_log));
    let git = Git::new(git_cmd);

    let f = File::open(YAML_FILE).unwrap();
    let template = Template::new(f).unwrap();
    let changelog = Changelog::new(template, git);

    let res = changelog.produce();
    assert!(res.is_err());
    assert!(res
        .unwrap_err()
        .to_string()
        .starts_with("Could not extract 'description' from commit message text"));
}
