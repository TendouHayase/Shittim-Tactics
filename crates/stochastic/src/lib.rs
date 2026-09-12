pub mod dist;

// 표현을 크레이트 밖으로 내보내지 않는다. 배포 전 격자 방식으로 갈아탈 때
// dist::HitBag의 시그니처만 유지되면 되도록 여기서 막아둔다.
pub(crate) mod pmf;
