

// template <class F>
// void PrintFst(const F& fst) {
//   for (fst::StateIterator<F> siter(fst); !siter.Done(); siter.Next())  {
//       auto state_id = siter.Value();
//       for (fst::ArcIterator<F> aiter(fst, state_id); !aiter.Done(); aiter.Next()) {
//         auto &arc = aiter.Value();
//         std::cout << state_id << "\t" << arc.nextstate << "\t" << arc.ilabel << "\t" << arc.olabel << "\t" << arc.weight << std::endl;
//       }
//       if (fst.Final(state_id) != F::Weight::Zero()) {
//         std::cout << state_id << "\t" << fst.Final(state_id) << std::endl;
//       }
//   }
// }

const uint64 kDoNotEncodeWeights = fst::kAcyclic | fst::kUnweighted | fst::kUnweightedCycles;

template <class Arc>
void OptimizeTransducer(fst::MutableFst<Arc> *fst, bool compute_props = false) {
  using Weight = typename Arc::Weight;
  // If the FST is not (known to be) epsilon-free, perform epsilon-removal.
  if (fst->Properties(fst::kNoEpsilons, compute_props) != fst::kNoEpsilons) {
    std::cout << "RmEpsilon" << std::endl;
    RmEpsilon(fst);
  }
  // Combines identically labeled arcs with the same source and destination,
  // and sums their weights.
  std::cout << "ArcSum" << std::endl;
  StateMap(fst, fst::ArcSumMapper<Arc>(*fst));
  // The FST has non-idempotent weights; limiting optimization possibilities.
  if (!(Weight::Properties() & fst::kIdempotent)) {
    std::cout << "W not idempotent" << std::endl;
    if (fst->Properties(fst::kIDeterministic, compute_props) != fst::kIDeterministic) {
      std::cout << "Not IDeterministic" << std::endl;
      // But "any acyclic weighted automaton over a zero-sum-free semiring has
      // the twins property and is determinizable" (Mohri 2006). We just have to
      // encode labels.
      if (fst->Properties(fst::kAcyclic, compute_props)) {
        std::cout << "Acyclic -> encode, determinize, minimimize and decode" << std::endl;
        fst::EncodeMapper<Arc> encoder(fst::kEncodeLabels, fst::ENCODE);
        Encode(fst, &encoder);
        {
          std::unique_ptr<fst::MutableFst<Arc>> tfst(fst->Copy());
          Determinize(*tfst, fst);
        }
        Minimize(fst);
        Decode(fst, encoder);
      }
    } else {
      std::cout << "IDeterministic -> Minimize" << std::endl;
      Minimize(fst);
    }
  } else {
    std::cout << "W idempotent" << std::endl;
    // If the FST is not (known to be) deterministic, determinize it.
    if (fst->Properties(fst::kIDeterministic, compute_props) != fst::kIDeterministic) {
      std::cout << "Not IDeterministic" << std::endl;
      // FST labels are always encoded before determinization and minimization.
      // If the FST is not known to have no weighted cycles, its weights are
      // also
      // encoded before determinization and minimization.
      if (!fst->Properties(kDoNotEncodeWeights, compute_props)) {
        {
          std::cout << "EncodeWeights and Labels -> encode, deter, min, decode, arcsum" << std::endl;
          fst::EncodeMapper<Arc> encoder(fst::kEncodeLabels | fst::kEncodeWeights, fst::ENCODE);
          Encode(fst, &encoder);
          {
            std::unique_ptr<fst::MutableFst<Arc>> tfst(fst->Copy());
            Determinize(*tfst, fst);
          }
          Minimize(fst);
          Decode(fst, encoder);
        }
        StateMap(fst, fst::ArcSumMapper<Arc>(*fst));
      } else {
        std::cout << "EncodeLabels -> encode, deter, min, decode" << std::endl;
        fst::EncodeMapper<Arc> encoder(fst::kEncodeLabels, fst::ENCODE);
        Encode(fst, &encoder);
        {
          std::unique_ptr<fst::MutableFst<Arc>> tfst(fst->Copy());
          Determinize(*tfst, fst);
        }
        Minimize(fst);
        Decode(fst, encoder);
      }
    } else {
      std::cout << "IDeterministic -> Minimize" << std::endl;
      Minimize(fst);
      std::cout << "After Minimize" << std::endl;
      // PrintFst(*fst);
    }
  }
}

// Generic FST optimization function to be used when the FST is known to be an
// acceptor.
template <class Arc>
void OptimizeAcceptor(fst::MutableFst<Arc> *fst, bool compute_props = false) {
  using fst::kIDeterministic;
  using fst::kEncodeWeights;
  using fst::kAcyclic;
  using fst::kIdempotent;
  using fst::MutableFst;
  using Weight = typename Arc::Weight;
  // If the FST is not (known to be) epsilon-free, perform epsilon-removal.
  MaybeRmEpsilon(fst, compute_props);
  // Combines identically labeled arcs with the same source and destination,
  // and sums their weights.
  ArcSumMap(fst);
  // TODO(kbg): Switch on the following at compile time.
  // The FST has non-idempotent weights; limiting optimization possibilities.
  if (!(Weight::Properties() & kIdempotent)) {
    if (fst->Properties(kIDeterministic, compute_props) != kIDeterministic) {
      // But "any acyclic weighted automaton over a zero-sum-free semiring has
      // the twins property and is determinizable" (Mohri 2006).
      if (fst->Properties(kAcyclic, compute_props) == kAcyclic) {
        DeterminizeAndMinimize(fst);
      }
    } else {
      Minimize(fst);
    }
  } else {
    // If the FST is not (known to be) deterministic, determinize it.
    if (fst->Properties(kIDeterministic, compute_props) != kIDeterministic) {
      // If the FST is not known to have no weighted cycles, it is encoded
      // before determinization and minimization.
      if (!fst->Properties(kDoNotEncodeWeights, compute_props)) {
        OptimizeAs(fst, kEncodeWeights);
        ArcSumMap(fst);
      } else {
        DeterminizeAndMinimize(fst);
      }
    } else {
      Minimize(fst);
    }
  }
}

namespace fst {
// Calls RmEpsilon if the FST is not (known to be) epsilon-free.
template <class Arc>
void MaybeRmEpsilon(fst::MutableFst<Arc> *fst, bool compute_props = false) {
  if (fst->Properties(fst::kNoEpsilons, compute_props) != fst::kNoEpsilons) {
    RmEpsilon(fst);
  }
}

// Optimizes the FST according to the encoder flags:
//
//   kEncodeLabels: optimize as a weighted acceptor
//   kEncodeWeights: optimize as an unweighted transducer
//   kEncodeLabels | kEncodeWeights: optimize as an unweighted acceptor
template <class Arc>
void OptimizeAs(MutableFst<Arc> *fst, uint32 flags) {
  EncodeMapper<Arc> encoder(flags, ENCODE);
  Encode(fst, &encoder);
  DeterminizeAndMinimize(fst);
  Decode(fst, encoder);
}

// Simulates determinization "in place".
template <class Arc>
void Determinize(MutableFst<Arc> *fst) {
  std::unique_ptr<MutableFst<Arc>> tfst(fst->Copy());
  Determinize(*tfst, fst);
}

template <class Arc>
void ArcSumMap(MutableFst<Arc> *fst) {
  StateMap(fst, ArcSumMapper<Arc>(*fst));
}

template <class Arc>
void DeterminizeAndMinimize(MutableFst<Arc> *fst) {
  Determinize(fst);
  Minimize(fst);
}

}
// This function performs a simple space optimization on FSTs that are
// (unions of) pairs of strings. It first pushes labels towards the initial
// state, then performs epsilon-removal. This will reduce the number of arcs
// and states by the length of the shorter of the two strings in the
// cross-product; label-pushing may also speed up downstream composition.
template <class Arc>
void OptimizeStringCrossProducts(fst::MutableFst<Arc> *fst,
                                 bool compute_props = false) {
  // Pushes labels towards the initial state.
  {
    std::unique_ptr<fst::MutableFst<Arc>> tfst(fst->Copy());
    fst::Push<Arc, fst::REWEIGHT_TO_INITIAL>(*tfst, fst, fst::kPushLabels);
  }
  MaybeRmEpsilon(fst, compute_props);
}

// Generic FST optimization function; use the more-specialized forms below if
// the FST is known to be an acceptor or a transducer.


template<class Arc>
int props(const fst::MutableFst<Arc> &fst) {
  auto res = fst.Properties(fst::kFstProperties, false);
  res -= fst::kExpanded;
  res -= fst::kMutable;
  return res;
}

// Destructive signature.
template <class Arc>
void Optimize(fst::MutableFst<Arc> *fst, bool compute_props = false) {
  using fst::kAcceptor;
  if (fst->Properties(kAcceptor, compute_props) != kAcceptor) {
    // The FST is (may be) a transducer.
    std::cout << "Optimize transducer" << std::endl;
    OptimizeTransducer(fst, compute_props);
  } else {
    std::cout << "Optimize acceptor" << std::endl;
    // The FST is (known to be) an acceptor.
    OptimizeAcceptor(fst, compute_props);
  }
}

// This function optimizes the right-hand side of an FST difference in an
// attempt to satisfy the constraint that it must be epsilon-free and
// deterministic. The input is assumed to be an unweighted acceptor.
template <class Arc>
void OptimizeDifferenceRhs(fst::MutableFst<Arc> *fst, bool compute_props = false) {
  // If the FST is not (known to be) epsilon-free, performs epsilon-removal.
  fst::MaybeRmEpsilon(fst, compute_props);
  // If the FST is not (known to be) deterministic, determinizes it; note that
  // this operation will not introduce epsilons as the input is an acceptor.
  if (fst->Properties(fst::kIDeterministic, compute_props) != fst::kIDeterministic) {
    Determinize(fst);
  }
  // Minimally, RHS must be input label-sorted; the LHS does not need
  // arc-sorting when the RHS is deterministic (as it now should be).
  fst::ILabelCompare<Arc> icomp;
  ArcSort(fst, icomp);
}