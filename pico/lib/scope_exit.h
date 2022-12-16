#ifndef TATA_LIB_SCOPE_EXIT
#define TATA_LIB_SCOPE_EXIT

template <typename EF> class scope_exit {
public:
  scope_exit(EF ef) : ef(ef) {}
  ~scope_exit() { ef(); }
  scope_exit operator=(scope_exit &o) = delete;

private:
  EF ef;
};

#endif // TATA_LIB_SCOPE_EXIT
