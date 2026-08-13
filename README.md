[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

# Shittim-Tactics

블루아카이브 총력전/대결전/제약해제결전 최적화 택틱 파인더

## 빌드

클론 직후에는 **먼저 생성기를 돌려야 합니다.**

```sh
cargo xtask
cargo build
```

`core`는 `students`/`bosses`에 의존할 수 없어서(순환 의존) 그쪽의 스킬·상태 정의를 xtask가
`core` 안으로 복제합니다. 그 복제본은 학생 수만큼 그대로 불어나므로 저장소에 두지 않고
`.gitignore`에 넣어두었습니다. 생성 대상은 다음과 같습니다.

- `crates/core/src/skills.rs`, `crates/core/src/skills/`
- `crates/core/src/states.rs`
- `crates/core/src/boss_macros.rs`
- `crates/core/src/skill_defs.rs`

돌리지 않으면 `file not found for module 'skills'`로 실패합니다. 원본은
`crates/students/src/skills/**`와 `crates/bosses/src/**`이며, 복제본을 고쳐도 다음 실행에서
사라집니다.

## TODO

- [x] A* 구현
- [ ] Beam Search 구현
- [ ] 모든 학생 스킬 구현
- [ ] 모든 보스 구현

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
