# catan_lib

Implémentation du jeu de société Catan en Rust : un moteur de règles sous forme de bibliothèque pure, et un client graphique construit dessus.

Le moteur est **déterministe** : à état égal et action égale, il produit toujours le même résultat. Tout le hasard — dés, mélange des tuiles, carte volée — est fourni par l'appelant, jamais tiré à l'intérieur. Cette propriété est volontaire : elle permet de faire tourner le même moteur sur un serveur et sur plusieurs clients sans divergence, et de rejouer une partie depuis son journal.

## Organisation

```
catan_lib/
├── engine/     bibliothèque de règles, sans UI ni réseau
└── client/     client graphique egui, natif ou WASM
```

Workspace Cargo. Le client dépend du moteur ; l'inverse n'est jamais vrai.

## État du projet

**Version 0.2.0** — une partie complète à 2–6 joueurs est jouable en local, de la mise en place à la victoire, dans une interface graphique.

### Implémenté

**Moteur**

- Génération de la topologie hexagonale (tuiles, sommets, arêtes et leurs adjacences), pour n'importe quelle forme de plateau
- Mise en place du plateau standard : répartition des terrains, pose des jetons en escargot, position initiale du voleur
- Détermination de l'ordre des joueurs par lancer de dés
- Phase de placement initial en serpentin, avec crédit des ressources de la deuxième colonie
- Production sur jet de dés, voleur inclus
- Construction de routes et de colonies, amélioration en villes, avec paiement atomique
- Résolution du 7 : défausses, déplacement du voleur, vol d'une carte
- Tour par tour, points de victoire, détection de la victoire

**Client**

- Rendu du plateau : terrains, jetons dimensionnés selon leur probabilité, voleur, routes et bâtiments
- Zoom à la molette, mise à l'échelle automatique de l'interface
- Sélection au clic des sommets, arêtes et tuiles, pilotée par la phase de jeu
- Main du joueur courant, panneau des joueurs, boutons de construction avec coût et disponibilité
- Modales de vol et de fin de partie

### Non implémenté

- Cartes développement
- Ports et échanges (entre joueurs comme avec la banque)
- Banque à réserve finie : les ressources sont actuellement créées sans limite
- Route la plus longue et armée la plus grande
- Limite de pièces par joueur (15 routes, 5 colonies, 4 villes)
- Plateaux autres que le standard : l'architecture les accepte, seules les données manquent
- Multijoueur en réseau — voir plus bas

## Principes de conception

Quatre décisions structurent le code et méritent d'être connues avant d'y toucher.

**Les états invalides sont inconstructibles.** Un désert ne peut pas porter de jeton, parce que la variante `Terrain::Desert` n'a pas de champ. Un jeton ne peut pas valoir 7, parce que son constructeur le refuse et que son champ est privé. La machine à états de la partie interdit de lancer les dés deux fois ou de poser deux colonies d'affilée pendant la mise en place. Le compilateur porte les règles partout où c'est possible.

**Le hasard vit à la périphérie.** Le moteur ne tire jamais au sort : il reçoit les jets de dés, l'agencement des tuiles et la carte volée. C'est ce qui rend les tests reproductibles et le multijoueur possible.

**Topologie et état sont séparés.** `Topology` contient la géométrie, figée à la création ; `Board` contient ce qui change. Sommets et arêtes ne sont jamais des objets mais des indices (`VertexId`, `EdgeId`), ce qui évite tout graphe de références et garde `Board` clonable et sérialisable.

**Vérifier puis appliquer.** Chaque règle existe en deux temps : une fonction pure qui valide et calcule (`can_place_road`, `production`), une fonction courte qui applique (`place_road`, `apply_roll`). Une action refusée ne modifie jamais l'état — un paiement impossible ne prélève rien. Le client suit le même patron : les panneaux collectent des `UiAction` sans rien muter, et l'application se fait en un seul point.

## Le multijoueur, en projet

Les décisions suivantes sont prises mais pas encore implémentées.

**On transmet les actions, pas l'état.** Messages minuscules, journal de partie gratuit, information cachée gérable. Cela repose entièrement sur le déterminisme du moteur.

**Deux protocoles distincts.** Un client envoie une intention sans hasard (« je lance les dés », « je vole le rouge ») ; le serveur la résout et diffuse un fait accompli (« le jet est 4 et 3 »). Ce ne sont donc pas les mêmes messages qui montent et qui descendent.

**Le serveur décrit ce qu'un client ne peut pas recalculer.** Poser une route se déduit du seul emplacement, puisque les coûts sont publics. La défausse et le vol, eux, doivent être annoncés — et de façon personnalisée, car un spectateur ne doit pas apprendre quelles cartes ont changé de main.

**Le client a une vue partielle.** Il connaît le plateau, sa propre main et le nombre de cartes des autres. Il peut donc vérifier ses propres coups avant de les envoyer, mais c'est le serveur qui fait autorité et qui rejette tout ce qui est illégal.

## Développement

```bash
cargo test              # 60 tests, moteur
cargo clippy
cargo run -p catan-client
```

Les tests se répartissent sur deux niveaux : les règles sont vérifiées unitairement au niveau de `Board` et `Topology`, sur des plateaux minimaux construits à la main, tandis que `game.rs` contient un test d'intégration qui joue une partie réelle du début à la fin sans jamais forcer un état.

Sur Ubuntu, le client demande quelques bibliothèques système :

```bash
sudo apt-get install libxcb-render0-dev libxcb-shape0-dev \
    libxcb-xfixes0-dev libxkbcommon-dev libssl-dev
```

Prérequis : Rust 1.85 ou plus récent (édition 2024).