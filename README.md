# Massa: Merkeziyetsiz ve Ölçeklendirilmiş Blok Zinciri

Massa, binlerce insan tarafından kontrol edilen gerçekten merkezi olmayan bir blok zinciridir. 
Çığır açan çok iş parçacıklı teknolojiyle toplu olarak benimsenmeye hazırız.

[![CI](https://github.com/massalabs/massa/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/massalabs/massa/actions/workflows/ci.yml?query=branch%3Amain)
[![Bors enabled](https://bors.tech/images/badge_small.svg)](https://app.bors.tech/repositories/39543)
[![Coverage Status](https://coveralls.io/repos/github/massalabs/massa/badge.svg?branch=main)](https://coveralls.io/github/massalabs/massa?branch=main)
[![Docs](https://img.shields.io/static/v1?label=docs&message=massa&color=blue)](https://massalabs.github.io/massa/massa_node/)
[![OpenRPC Playground](https://img.shields.io/static/v1?label=openrpc-playground&message=massa&color=blue)](https://playground.open-rpc.org/?schemaUrl=https://raw.githubusercontent.com/massalabs/massa/main/docs/technical-doc/openrpc.json)
[![Open in Gitpod](https://shields.io/badge/Gitpod-contribute-brightgreen?logo=gitpod&style=flat)](https://gitpod.io/#https://github.com/massalabs/massa)

## Giriş

[Massa](https://massa.net), merkezi olmayan bir ağda yüksek işlem hacmine ulaşan yeni bir blok zinciridir. 
Araştırmamız bu [teknik belgede](https://arxiv.org/pdf/1803.09029) yayınlanmıştır.
Binlerce düğüme sahip tamamen merkezi olmayan bir ağda bile saniyede 10.000 işlem hacmine ulaşıldığını gösteriyor.

Videolarla birlikte okunması kolay bir blog yazısı tanıtımı (https://massa.net/blog/introduction/) yazılmıştır

We are now releasing the **Massa testnet** in this Gitlab repository,
with its explorer available at <https://test.massa.net>.

Şimdi <https://test.massa.net> adresinde bulunan gezgin ile 
bu Gitlab deposunda **Massa test ağını** yayınlıyoruz.

## Testnet Teşvikleri

Merkeziyetçilikten uzaklaşma temel değerimiz olduğundan, bir Massa düğümü başlatmanıza ve çalıştırmanıza yardımcı olmak istiyoruz.
Test ağı aşamasında bir düğüm çalıştırmak ayrıca hataları bulmamıza ve kullanılabilirliği geliştirmemize yardımcı olur, 
bu nedenle ana ağ açılışında gerçek Massa ile ödüllendirilecektir.

Bu ödüllerin mekanizması [testnet kurallarında](https://massa.readthedocs.io/en/latest/testnet/rewards.html) açıklanmıştır.

## Testnet Tartışmaları

Testnet tartışmaları için lütfen [Discord](https://discord.com/invite/massa) sayfamızda testnet kanalına gelin.

Proje duyuruları için ağırlıklı olarak [Telegram](https://t.me/massanetwork) 
ve ayrıca bir [Twitter](https://twitter.com/MassaLabs) hesabımız var.

## Testnet'e katılmak için öğreticiler

-   [Node yükleme](https://massa.readthedocs.io/en/latest/testnet/install.html)
-   [Node çalıştırma](https://massa.readthedocs.io/en/latest/testnet/running.html)
-   [Cüzdan oluşturma](https://massa.readthedocs.io/en/latest/testnet/wallet.html)
-   [Stake etme](https://massa.readthedocs.io/en/latest/testnet/staking.html)
-   [Yönlendirilebilirlik eğitimi](https://massa.readthedocs.io/en/latest/testnet/routability.html) (İsteğe bağlı)
-   [Testnet ödül programı](https://massa.readthedocs.io/en/latest/testnet/rewards.html) (İsteğe bağlı)

## Diğer öğreticiler

-   [Node güncelleme](https://massa.readthedocs.io/en/latest/testnet/update.html)
-   [Topluluktan öğreticiler ve kaynaklar](https://massa.readthedocs.io/en/latest/testnet/community-resources.html)

## [SSS](https://massa.readthedocs.io/en/latest/testnet/faq.html) ve Sorun Giderme

[SSS](https://massa.readthedocs.io/en/latest/testnet/faq.html) bölümünde Massa protokolüyle ilgili sık karşılaşılan sorunlara ve soruların yanıtlarını bulacaksınız.

[Discord](https://discord.com/invite/massa) testnet kanalında soru sormaktan çekinmeyin.

## Teknik belgeler

-   [API](https://massa.readthedocs.io/en/latest/technical-doc/api.html)
-   [Eş Zamanlılık](https://massa.readthedocs.io/en/latest/technical-doc/concurrency.html)
-   [Akıllı sözleşme VM blok besleme süreci](https://massa.readthedocs.io/en/latest/technical-doc/vm-block-feed.html)
-   [Sanal makine defteri etkileşimi](https://massa.readthedocs.io/en/latest/technical-doc/vm-ledger-interaction.html)
-   [Sahte ağ oluşturma](https://massa.readthedocs.io/en/latest/technical-doc/dummy-network-generation.html)
