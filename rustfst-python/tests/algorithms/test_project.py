from rustfst import VectorFst, Tr
from rustfst.algorithms.project import ProjectType


def test_project_input():
    # FST 1
    fst1 = VectorFst()

    s1 = fst1.add_state()
    s2 = fst1.add_state()
    s3 = fst1.add_state()

    fst1.set_start(s1)
    fst1.set_final(s3)

    tr1_1 = Tr(1, 2, 1.0, s2)
    fst1.add_tr(s1, tr1_1)

    tr1_2 = Tr(3, 4, 2.0, s2)
    fst1.add_tr(s1, tr1_2)

    tr1_3 = Tr(4, 5, 3.0, s3)
    fst1.add_tr(s2, tr1_3)

    # Expected FST
    expected_fst = VectorFst()

    s1 = expected_fst.add_state()
    s2 = expected_fst.add_state()
    s3 = expected_fst.add_state()

    expected_fst.set_start(s1)
    expected_fst.set_final(s3)

    tr1_1 = Tr(1, 1, 1.0, s2)
    expected_fst.add_tr(s1, tr1_1)

    tr1_2 = Tr(3, 3, 2.0, s2)
    expected_fst.add_tr(s1, tr1_2)

    tr1_3 = Tr(4, 4, 3.0, s3)
    expected_fst.add_tr(s2, tr1_3)

    fst2 = fst1.project()

    assert expected_fst == fst2


def test_project_output():
    # FST 1
    fst1 = VectorFst()

    s1 = fst1.add_state()
    s2 = fst1.add_state()
    s3 = fst1.add_state()

    fst1.set_start(s1)
    fst1.set_final(s3)

    tr1_1 = Tr(1, 2, 1.0, s2)
    fst1.add_tr(s1, tr1_1)

    tr1_2 = Tr(3, 4, 2.0, s2)
    fst1.add_tr(s1, tr1_2)

    tr1_3 = Tr(4, 5, 3.0, s3)
    fst1.add_tr(s2, tr1_3)

    # Expected FST
    expected_fst = VectorFst()

    s1 = expected_fst.add_state()
    s2 = expected_fst.add_state()
    s3 = expected_fst.add_state()

    expected_fst.set_start(s1)
    expected_fst.set_final(s3)

    tr1_1 = Tr(2, 2, 1.0, s2)
    expected_fst.add_tr(s1, tr1_1)

    tr1_2 = Tr(4, 4, 2.0, s2)
    expected_fst.add_tr(s1, tr1_2)

    tr1_3 = Tr(5, 5, 3.0, s3)
    expected_fst.add_tr(s2, tr1_3)

    project_type = ProjectType.PROJECT_OUTPUT
    fst2 = fst1.project(project_type)

    assert expected_fst == fst2
