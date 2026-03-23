# AOXChain Testnet + Mainnet Hizalı 9 Etap Programı + Stage 0 Altyapı Tamamlama Planı (TR)

Bu belge, AOXChain için **testnet ve mainnet hazırlığını ayrı yol haritaları olarak değil, tek hizalı yürütme programı olarak** tanımlar.

Bu revizyonda programın önüne zorunlu bir **Stage 0 / Başlangıç Bazı** eklenmiştir. Kullanılacak başlangıç etiketi: **`v0.1.1-alpha`**. Bu baseline tamamlanmadan Etap 1 başlamaz.

Amaç sadece bir testnet açmak değildir. Amaç, testnet üzerinde doğrulanan her kritik kabiliyetin kontrollü biçimde mainnet hazırlık kapısına bağlanmasıdır. Böylece:

- testnet, “demo ortamı” olarak kalmaz,
- mainnet, test edilmemiş varsayımlarla başlatılmaz,
- her etap için aynı anda **testnet çıkışı** ve **mainnet karşılığı** tanımlanır,
- teknik ekip, operasyon ekibi ve güvenlik inceleme akışı aynı program içinde hizalanır.

Bu programın temel dayanakları şunlardır:

- gerçek ağ için gerekli zorunlu kapılar,
- audit-readiness ve operasyonel sertleştirme gereksinimleri,
- consensus kernel için testnet öncesi ve mainnet öncesi kapanış maddeleri,
- readiness evidence modelinde blocker olarak tanımlanan alanlar.

---

## 1. Programın yönetim ilkeleri

### 1.1 Tek backlog, çift hedef
Her iş kalemi tek backlog içinde tutulur; ancak her kalem için iki ayrı sonuç alanı yazılır:

1. **Testnet Outcome**
2. **Mainnet Outcome**

Örnek:
- Testnet outcome: 5 node ayrı hostlarda propagation raporu üretildi.
- Mainnet outcome: aynı ölçüm hattı release gate ve SLO eşiğine bağlandı.

### 1.2 “Testnet geçti” demek “mainnet hazır” demek değildir
Her etapta testnet kabul kriteri ile mainnet kabul kriteri ayrı yazılmalıdır.

- Testnet kabulü: özelliğin gerçek ağda çalıştığının kanıtı.
- Mainnet kabulü: özelliğin güvenlik, operasyon ve rollback gereksinimleriyle birlikte sürdürülebilir olduğunun kanıtı.

### 1.3 Blocker mantığı
Aşağıdaki alanlar blocker olarak ele alınmalıdır:

- multi-host real network validation,
- partition / fault scenarios,
- state sync and snapshot recovery,
- soak test.

Bu alanlardan herhangi biri eksikse program “public real testnet ready” etiketi vermez.

### 1.4 Kanıt üretmeden etap kapanmaz
Her etap için aşağıdakiler zorunludur:

- dokümantasyon güncellemesi,
- çalıştırılabilir komut seti,
- ölçüm/kanıt artifact’ı,
- açık risk listesi,
- bir sonraki etabı unblock eden çıktı.

### 1.5 Stage 0 zorunlu başlangıç kuralı
Etap 1'e geçmeden önce aşağıdaki alanlar **%100 tamamlanmış** kabul edilmelidir:

- docs yapısı eksiksiz ve birbiriyle tutarlı olmalı,
- `SUMMARY.md` ve mdBook gezinmesi eksiksiz olmalı,
- sürüm etiketi ve başlangıç release notu tanımlanmış olmalı,
- eksik altyapı dosyaları, klasörleri, örnek config'leri ve placeholder yerine gerçek içerikler tamamlanmış olmalı,
- hangi dosyanın neden var olduğu, nerede bulunduğu ve kim tarafından sahiplenildiği yazılmış olmalı.

Bu programda bu başlangıç bazı **`v0.1.1-alpha`** olarak adlandırılır. Amaç: **altyapı %100**.

---

## 2. Program sürümü ve etap özeti

### Başlangıç baseline sürümü
- **Program başlangıç sürümü:** `v0.1.1-alpha`
- **Baseline adı:** Stage 0 / Altyapı %100 Tamamlama Bazı
- **Kural:** Stage 0 kapanmadan Etap 1 başlamaz.

### Stage 0 — Etap 1 öncesi zorunlu baz

#### Hedef
Belge, mdBook, sürümleme ve eksik altyapı dosyaları tarafında boşluk bırakmadan tek bir başlangıç tabanı oluşturmak.

#### Testnet çıktısı
- testnet için gerekli tüm temel docs, config örnekleri, çalışma akışı açıklamaları ve klasör yerleri açık biçimde yazılır,
- eksik veya placeholder kalan markdown/dosya referansları temizlenir,
- operatör bir dosyanın ne işe yaradığını belgeye bakarak anlayabilir.

#### Mainnet çıktısı
- aynı temel yapı mainnet hazırlık belgelerinde de tam ve tutarlı hale gelir,
- sürüm etiketi, başlangıç release notu ve belge sahipliği netleştirilir,
- dokümantasyon ve release dili, launch gate'e bağlanabilecek kadar düzenli hale getirilir.

#### İş kalemleri
- `docs/src/` altında eksik veya belirsiz başlıkları tamamla,
- `SUMMARY.md` içindeki gezinmeyi eksiksiz hale getir,
- mdBook kırık link, eksik sayfa ve yanlış başlıklarını temizle,
- `v0.1.1-alpha` için başlangıç kapsam notu yaz,
- eksik altyapı dosyaları için gerçek içerik üret veya açıkça neden eksik olduğunu backlog'a bağla,
- config / fixture / artifact klasör yerlerini belge üstünde tek tek yaz,
- sahiplik matrisi oluştur: docs owner, release owner, infra owner, network owner.

#### Exit criteria
- docs ve mdBook tarafında bilinen kırık gezinme veya eksik sayfa kalmaz,
- başlangıç sürümü `v0.1.1-alpha` olarak kayda geçer,
- eksik altyapı dosyaları listelenmiş, yerleştirilmiş veya resmî backlog'a bağlanmış olur,
- “amacımız altyapı %100” ifadesi somut checklist'e çevrilmiş olur.

---

| Etap | Başlık | Testnet odağı | Mainnet hizası |
|---|---|---|---|
| 1 | Program ve mimari hizalama | kapsam ve exit criteria | release governance ve sorumluluk matrisi |
| 2 | Gerçek node çalıştırma temeli | node-run servis akışı | production service lifecycle |
| 3 | Dağıtık ağ ve P2P gerçekleme | 3-5 node ayrı host | production transport + peer policy |
| 4 | Consensus güvenlik çekirdeği | testnet öncesi consensus blocker’ları | mainnet safety invariants |
| 5 | Dayanıklılık ve recovery | partition/restart/rejoin/snapshot | crash consistency + rollback discipline |
| 6 | RPC ve public yüzey hardening | güvenli testnet erişimi | production authn/authz/TLS/rate limit |
| 7 | Gözlemlenebilirlik ve soak | uzun süreli testnet koşusu | SLO/SLA ve alarm politikaları |
| 8 | Release, upgrade ve provenance | testnet upgrade provası | signed release + migration + rollback |
| 9 | Launch gate ve kademeli açılış | public testnet launch kararı | controlled mainnet launch kararı |


## 3. Etap 1 — Program ve mimari hizalama

### Hedef
Teknik, operasyonel ve güvenlik backlog’larını tek program altında toplamak.

### Testnet çıktısı
- testnet için gerekli blocker listesi dondurulur,
- etap bazlı sahiplik atanır,
- readiness evidence dosyası stage review’larda kullanılacak tek kaynak olur.

### Mainnet çıktısı
- release owner, security owner, networking owner, consensus owner, ops owner netleştirilir,
- mainnet exception süreci tanımlanır,
- “hangi eksikler testnet’i durdurur, hangileri mainnet’i durdurur” ayrımı yazılı hale gelir.

### İş kalemleri
- ortak risk register oluştur,
- stage review şablonu tanımla,
- exit criteria tablosunu standardize et,
- doc-owner ve runbook-owner atamalarını yap.

### Exit criteria
- 9 etap için sahipler atanmış,
- her etap için testnet/mainnet kabul kriteri yazılmış,
- blocker ve non-blocker ayrımı repo belgeleriyle uyumlu.

---

## 4. Etap 2 — Gerçek node çalıştırma temeli

### Hedef
Local smoke komutlarından çıkıp sürekli çalışan, denetlenebilir node servis modeline geçmek.

### Testnet çıktısı
- tek komutla node home bootstrap + node-run yapılır,
- health/readiness sinyalleri üretilir,
- süreç yeniden başlatıldığında deterministik boot davranışı gözlenir.

### Mainnet çıktısı
- systemd/container/k8s benzeri servis modeli için lifecycle tanımı yapılır,
- log rotation, env var sözleşmesi, secrets sınırı ve restart davranışı dokümante edilir.

### İş kalemleri
- `node-run` servis akışını birincil çalışma modu yap,
- health ve readiness endpoint/komutlarını standardize et,
- startup öncesi config/genesis/identity doğrulama kapıları ekle,
- servis runbook’unu yaz.

### Exit criteria
- en az 1 node, sürekli modda istikrarlı çalışır,
- operatör tek-atımlık smoke yerine servis akışını kullanır,
- servis başlatma/durdurma/doğrulama runbook’u tamamlanır.

---

## 5. Etap 3 — Dağıtık ağ ve P2P gerçekleme

### Hedef
Loopback smoke’tan çıkıp gerçek çok düğümlü ağ davranışını doğrulamak.

### Testnet çıktısı
- 3-5 node ayrı host veya ayrı network namespace üzerinde çalışır,
- peer bağlantısı ve block/tx propagation ölçülür,
- multi-host validation raporu oluşturulur.

### Mainnet çıktısı
- transport-backed gossip tamamlanır,
- peer admission, routing ve secure-mode varsayımları production profile ile uyumlu hale gelir,
- public topology için peer policy yazılır.

### İş kalemleri
- gerçek transport bağla,
- peer routing/discovery yaklaşımını netleştir,
- propagation metriği üret,
- distributed validation artifacts klasörünü standardize et.

### Exit criteria
- multi-host validation `missing` durumundan çıkar,
- 3+ node ağ raporu tekrar üretilebilir hale gelir,
- peer sayısı, sync yakınsaması ve propagation görünürlüğü raporlanır.

---

## 6. Etap 4 — Consensus güvenlik çekirdeği

### Hedef
Deterministic scaffold’u testnet ve ardından mainnet için güvenlik odaklı consensus çekirdeğine yükseltmek.

### Testnet çıktısı
- quorum certificate modeli uygulanır,
- validator-set snapshots eklenir,
- persistent consensus store hazırlanır,
- replay/recovery path çalışır,
- authenticated transport envelope ve temel property/fuzz testleri eklenir.

### Mainnet çıktısı
- safety invariant’leri belge ve testlerle bağlanır,
- certificate bağlamı era/round/validator-set/signer set seviyesinde doğrulanır,
- consensus değişiklikleri için rollback ve compatibility kapıları hazırlanır.

### İş kalemleri
- QC veri modeli ve doğrulama hattı,
- validator authority snapshot akışı,
- persistent store ve replay log tasarımı,
- pacemaker genişletmesi,
- consensus integration + adversarial test matrisi.

### Exit criteria
- “must fix before testnet” listesi kapatılmış olur,
- consensus state restart sonrası deterministik davranır,
- quorum/finality yolunda tekrar üretilebilir test kanıtı vardır.

---

## 7. Etap 5 — Dayanıklılık ve recovery

### Hedef
Ağın hata, partition ve yeniden katılım senaryolarında güvenli ve ölçülebilir davranmasını sağlamak.

### Testnet çıktısı
- partition/restart/delay/drop/timeout senaryoları çalıştırılır,
- snapshot export/import ve node rejoin testi yapılır,
- recovery süresi ve state hash tutarlılığı ölçülür.

### Mainnet çıktısı
- crash consistency, disk bozulması, rollback ve restore disiplini yazılı hale gelir,
- recovery tatbikatı release öncesi zorunlu prova olur.

### İş kalemleri
- `tc netem`/firewall/process-control tabanlı test harness,
- snapshot formatı ve metadata sözleşmesi,
- restore sonrası hash/height doğrulaması,
- failure report şablonu ve recovery checklist’i.

### Exit criteria
- partition/fault ve snapshot/recovery blocker’ları kapanır,
- rejoin sonrası zincir yakınsaması kanıtlanır,
- recovery runbook’u operasyon ekibi tarafından uygulanabilir hale gelir.

---

## 8. Etap 6 — RPC ve public yüzey hardening

### Hedef
Testnet’i dış dünyaya açarken public attack surface’i kontrol altına almak.

### Testnet çıktısı
- güvenli testnet RPC profili tanımlanır,
- TLS/mTLS, auth, rate-limit ve erişim politikası uygulanır,
- insecure-mode davranışının nerede kabul edilemez olduğu belgelenir.

### Mainnet çıktısı
- production authn/authz modeli finalize edilir,
- certificate binding, replay kontrolü ve handshake güvenlik varsayımları denetlenir,
- public endpoint policy ve abuse response planı yazılır.

### İş kalemleri
- JSON-RPC / WS / gRPC için güvenlik profilleri,
- IP / client / method bazlı rate-limit,
- certificate ve attestation kontrolleri,
- public API runbook ve incident response ekleri.

### Exit criteria
- güvenli testnet erişimi operasyonel olarak açılabilir,
- network security checklist maddeleri ölçülebilir hale gelir,
- public endpoint’ler için kabul/ret kuralları açıktır.

---

## 9. Etap 7 — Gözlemlenebilirlik ve soak

### Hedef
Ağın kısa demo yerine uzun süreli işletime uygun olduğunu göstermek.

### Testnet çıktısı
- soak test planı uygulanır,
- block time, throughput, peer count, sync state, error counters görünür olur,
- uzun süreli çalışma boyunca stall/leak/crash sinyalleri toplanır.

### Mainnet çıktısı
- SLO/SLA eşiği belirlenir,
- alarm kuralları ve escalation akışı tanımlanır,
- release gate’e bağlanan telemetry dashboard seti hazırlanır.

### İş kalemleri
- standard metric seti,
- tracing/structured logs sözleşmesi,
- soak runner ve artifact formatı,
- alert routing ve on-call entegrasyonu.

### Exit criteria
- soak test blocker’ı kapanır,
- telemetry `partial` durumundan çıkar,
- belirli süreli koşu için sağlık raporu üretilebilir hale gelir.

---

## 10. Etap 8 — Release, upgrade ve provenance

### Hedef
Ağ yazılımının güvenilir biçimde yayınlanması ve yükseltilmesi.

### Testnet çıktısı
- testnet için imzalı artifact akışı kurulur,
- upgrade rehearsal yapılır,
- migration/compatibility/rollback provası yapılır.

### Mainnet çıktısı
- signed release manifests,
- reproducible build attestations,
- deterministic upgrade/version migration policy,
- rollback gate ve release approval süreci tamamlanır.

### İş kalemleri
- artifact signing,
- version compatibility matrisi,
- schema/state migration testleri,
- rollback prosedürü ve release checklist.

### Exit criteria
- upgrade/migration planning `missing` durumundan çıkar,
- en az bir testnet upgrade provası başarıyla tamamlanır,
- release provenance zinciri doğrulanabilir hale gelir.

---

## 11. Etap 9 — Launch gate ve kademeli açılış

### Hedef
Testnet ve mainnet için bağımsız fakat hizalı launch kararı verebilmek.

### Testnet çıktısı
- controlled public testnet launch kararı verilir,
- validator/operator onboarding seti tamamlanır,
- launch sonrası ilk 2-4 hafta için risk odaklı gözlem planı çalışır.

### Mainnet çıktısı
- mainnet launch gate yalnızca tüm zorunlu kapılar kapalıysa açılır,
- istisna varsa time-boxed ve release owner onaylı olur,
- progressive rollout + rollback kriterleri önceden ilan edilir.

### İş kalemleri
- launch review board toplantısı,
- exception register gözden geçirme,
- canary/progressive rollout şeması,
- first-week incident and escalation drill.

### Exit criteria
- testnet launch raporu ve mainnet readiness raporu birbirine bağlanır,
- her iki ağ için ayrı ama hizalı go/no-go kararı kayıt altına alınır,
- post-launch takip penceresi sahipleri atanır.

---

## 12. Etap bağımlılıkları

### Sıralı bağımlılıklar
- Etap 2, Etap 1 olmadan başlamamalı.
- Etap 3, Etap 2’nin servis akışı netleşmeden kapanmamalı.
- Etap 4, Etap 3’te gerçek ağ hareketi görülmeden production-iddialı kapanmamalı.
- Etap 5, Etap 4’te replay/persistent-store tabanı kurulmadan güvenli ilerlemez.
- Etap 6 ve 7, Etap 3–5 çıktıları üstüne inşa edilir.
- Etap 8, Etap 4–7’den gelen teknik ve operasyonel kanıt olmadan anlamsız kalır.
- Etap 9, önceki bütün etapların kapanış raporunu tüketir.

### Paralel yürütülebilecek akışlar
- Etap 6 ile Etap 7 kısmen paralel yürütülebilir.
- Etap 8’in release otomasyonu, Etap 6–7 sonlarına doğru başlatılabilir.
- Etap 1 boyunca risk, doc ve sahiplik işleri tüm etaplara paralel akar.

---

## 13. Her etap için zorunlu çıktı paketi

Her etap kapanırken aşağıdaki paket üretilmelidir:

1. **Design note**
2. **Runbook update**
3. **Programmatic evidence**
4. **Observed risk / residual risk listesi**
5. **Testnet outcome**
6. **Mainnet outcome**
7. **Go / No-Go kararı**

Artifact örnekleri:
- benchmark logları,
- distributed validation json’ları,
- soak raporları,
- screenshot yerine terminal transcript’leri,
- signed manifest hash kayıtları,
- failure injection sonuç raporları.

---

## 14. Önerilen yönetim ritmi

### Haftalık
- engineering stage sync,
- blocker review,
- dokümantasyon delta kontrolü,
- risk register güncellemesi.

### Etap sonu
- exit criteria review,
- artifacts doğrulaması,
- açık risklerin sahiplenilmesi,
- bir sonraki etap için unblock kararı.

### Launch öncesi
- full readiness review,
- incident drill tekrar kontrolü,
- runbook walk-through,
- release artifact doğrulaması.

---

## 15. Başarı ölçütü

Bu 9 etap programı, aşağıdaki durum elde edildiğinde başarılı kabul edilir:

- testnet, gerçek çok düğümlü ve gözlemlenebilir bir ağ olarak çalışır,
- testnet’te doğrulanan kritik yollar mainnet launch gate’ine ölçülebilir şekilde bağlanır,
- consensus, network, recovery, security ve ops alanlarında blocker kalmaz,
- release ve upgrade süreçleri imzalı, tekrar üretilebilir ve rollback destekli hale gelir,
- testnet ve mainnet artık ayrı hayaller değil, tek bir mühendislik programının iki farklı açılış kapısı olur.

---

## 16. Önerilen ilk kullanım biçimi

Bu belge aşağıdaki belgelerle birlikte kullanılmalıdır:

- `GERCEK_AG_HAZIRLIK_KRITERLERI_TR.md`
- `REAL_NETWORK_VALIDATION_RUNBOOK_TR.md`
- `AOXC_KERNEL_HARDENING_MASTER_PLAN_TR.md`
- `AUDIT_READINESS_AND_OPERATIONS.md`
- `MAINNET_READINESS_CHECKLIST.md`
- `models/mainnet_readiness_evidence_v1.yaml`

Uygulama önerisi:
1. Bu 9 etap belgesini ana program belgesi kabul et.
2. Her etap için issue/milestone aç.
3. Readiness evidence dosyasını etap review toplantılarında güncelle.
4. Testnet ve mainnet launch kararlarını bu belge üstünden ver.
